use embedded_graphics::{pixelcolor::Rgb565, prelude::WebColors};
use embedded_graphics_framebuf::FrameBuf;
use esp_idf_hal::{delay::Delay, peripherals::Peripherals};
use esp_idf_hal::adc::oneshot::{AdcDriver, AdcChannelDriver, config::AdcChannelConfig};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_hal::gpio::{Output, PinDriver, Pull};
use esp_idf_svc::wifi::EspWifi;

use crate::cardputer_hal::{
    input::{keyboard::{InputState, PressedSymbol}, keyboard_io::{CardputerKeyboard, Scancode, KeyEvent}},
    screen::{cardputer_screen::CardputerScreen, display::CardputerDisplay, framebuffer::CardputerFramebuffer},
    sd::cardputer_sd::CardputerSd,
    wifi::wifi::{CardWorderWifi, WifiConfig}};

#[derive(Clone, Copy)]
pub struct KeyboardState {
    pub key: Option<(KeyEvent, Scancode)>,
    pub input_state: InputState,
    pub pressed: Option<(KeyEvent, PressedSymbol)>,
}

/// Battery ADC reader — type-erased wrapper to avoid complex generics
pub struct BatteryReader {
    read_fn: Box<dyn FnMut() -> u16 + Send>,
}

impl BatteryReader {
    /// Read battery voltage in mV (averaged 64 samples, ×2 for divider)
    /// Note: reads the StampS3 internal battery only (120mAh).
    /// The base battery (1400mAh) connects via pogo pins but may not be
    /// visible at this measurement point.
    pub fn read_mv(&mut self) -> u32 {
        let mut sum: u32 = 0;
        for _ in 0..64 {
            sum += (self.read_fn)() as u32;
        }
        let adc_mv = sum / 64;
        let bat_mv = adc_mv * 2;
        log::info!("battery: adc_mv={}, bat_mv={}", adc_mv, bat_mv);
        bat_mv
    }

    pub fn read_percent(&mut self) -> u8 {
        battery_mv_to_percent(self.read_mv())
    }
}

/// All hardware components returned by `build_all`, ready to distribute across tasks.
pub struct CardputerParts<'a> {
    pub keyboard: CardputerKeyboard<'a>,
    pub display: CardputerDisplay<'a>,
    pub framebuffer: FrameBuf<Rgb565, CardputerFramebuffer>,
    pub hal: CardputerHal<'a>,
    pub battery: BatteryReader,
}

pub use cardworder_core::types::{DirStats, QuickStats};

/// Slim HAL that owns SD + Wi-Fi (lazy). Used on Core 0.
pub struct CardputerHal<'a> {
    sd: &'a mut CardputerSd<'a, Delay>,
    /// WiFi driver — created lazily on first StartWifi to save ~50KB heap
    wifi: Option<CardWorderWifi<'a>>,
    /// Modem peripheral — stored until WiFi is needed
    modem: Option<esp_idf_hal::modem::Modem<'a>>,
    /// System event loop — needed to create EspWifi
    sysloop: EspSystemEventLoop,
}

impl<'a> CardputerHal<'a> {
    /// Build all hardware and return parts for distribution to tasks.
    pub fn build_all(
        peripherals: Peripherals,
        sysloop: EspSystemEventLoop,
    ) -> CardputerParts<'a> {
        log::info!("hal: build_all — start");

        log::info!("hal: 3a CardputerScreen::build (SPI2 + display) …");
        let screen = CardputerScreen::build(
            Rgb565::CSS_BLACK,
            peripherals.spi2,
            peripherals.pins.gpio36,
            peripherals.pins.gpio35,
            peripherals.pins.gpio37,
            peripherals.pins.gpio34,
            peripherals.pins.gpio33,
            peripherals.pins.gpio38,
        );
        log::info!("hal: 3a CardputerScreen::build — done");

        log::info!("hal: 3b CardputerSd::build (SPI3 + SD) …");
        // Box::leak immediately — ESP-IDF SPI DMA has internal state that breaks if the struct moves
        let sd: &'a mut CardputerSd<'a, Delay> = unsafe {
            std::mem::transmute(Box::leak(Box::new(CardputerSd::build(
                peripherals.spi3,
                peripherals.pins.gpio40,
                peripherals.pins.gpio39,
                peripherals.pins.gpio14,
                peripherals.pins.gpio12,
            ))))
        };
        log::info!("hal: 3b CardputerSd::build — done");

        log::info!("hal: 3c keyboard mux GPIO (8,9,11) …");
        let mux_pins: [PinDriver<'_, Output>; 3] = [
            PinDriver::output(peripherals.pins.gpio8.degrade_output()).unwrap(),
            PinDriver::output(peripherals.pins.gpio9.degrade_output()).unwrap(),
            PinDriver::output(peripherals.pins.gpio11.degrade_output()).unwrap(),
        ];
        log::info!("hal: 3c mux pins — done");

        log::info!("hal: 3d keyboard column GPIO (13,15,3–7) …");
        let column_pins = [
            PinDriver::input(peripherals.pins.gpio13.degrade_input_output(), Pull::Up).unwrap(),
            PinDriver::input(peripherals.pins.gpio15.degrade_input_output(), Pull::Up).unwrap(),
            PinDriver::input(peripherals.pins.gpio3.degrade_input_output(), Pull::Up).unwrap(),
            PinDriver::input(peripherals.pins.gpio4.degrade_input_output(), Pull::Up).unwrap(),
            PinDriver::input(peripherals.pins.gpio5.degrade_input_output(), Pull::Up).unwrap(),
            PinDriver::input(peripherals.pins.gpio6.degrade_input_output(), Pull::Up).unwrap(),
            PinDriver::input(peripherals.pins.gpio7.degrade_input_output(), Pull::Up).unwrap(),
        ];
        log::info!("hal: 3d column pins — done");

        log::info!("hal: 3e CardputerKeyboard::new + init …");
        let mut keyboard = CardputerKeyboard::new(mux_pins, column_pins);
        keyboard.init();
        log::info!("hal: 3e keyboard — done");

        // WiFi is NOT created at boot — deferred until Core0Action::StartWifi
        // This saves ~50KB heap when WiFi is not in use
        log::info!("hal: 3f WiFi modem stored (lazy init)");

        // Battery ADC (GPIO10 via 0.5x resistor divider)
        log::info!("hal: 3g ADC for battery …");
        let adc = AdcDriver::new(peripherals.adc1).unwrap();
        let adc_config = AdcChannelConfig {
            attenuation: esp_idf_hal::adc::attenuation::DB_11,
            calibration: esp_idf_hal::adc::oneshot::config::Calibration::Curve,
            ..AdcChannelConfig::new()
        };
        let mut bat_channel = AdcChannelDriver::new(adc, peripherals.pins.gpio10, &adc_config).unwrap();
        log::info!("hal: 3g ADC — done");

        let (display, framebuffer) = screen.into_parts();

        log::info!("hal: build_all — complete");
        CardputerParts {
            keyboard,
            display,
            framebuffer,
            battery: BatteryReader {
                read_fn: Box::new(move || bat_channel.read().unwrap_or(0)),
            },
            hal: CardputerHal {
                sd,
                wifi: None,
                modem: Some(peripherals.modem),
                sysloop,
            },
        }
    }

    pub fn create_wifi_file_if_non_exists(
        &mut self,
        ssid: heapless::String<32>,
        password: heapless::String<64>,
    ) -> anyhow::Result<()> {
        let is_file_exists = { self.sd.is_file_exists("wifi_cfg.jsn").unwrap() };
        if !is_file_exists {
            let config = WifiConfig { ssid, password };
            let config_str = serde_json::to_string(&config).unwrap();
            self.sd
                .write_file("wifi_cfg.jsn", &config_str)
                .unwrap();
        }
        Ok(())
    }

    pub fn load_wifi_config(&mut self) -> anyhow::Result<WifiConfig> {
        let config_str = self
            .sd
            .read_file("wifi_cfg.jsn")
            .map_err(|_e| anyhow::anyhow!("Failed to read wifi_cfg.jsn"))?;

        let config: WifiConfig = serde_json::from_str(&config_str)?;

        Ok(config)
    }

    /// Lazily create EspWifi from modem if not already created.
    fn ensure_wifi(&mut self) -> anyhow::Result<()> {
        if self.wifi.is_none() {
            let modem = self.modem.take()
                .ok_or_else(|| anyhow::anyhow!("Modem already taken"))?;
            log::info!("hal: creating EspWifi (lazy init)");
            let esp_wifi = EspWifi::new(modem, self.sysloop.clone(), None)
                .map_err(|e| anyhow::anyhow!("EspWifi::new failed: {:?}", e))?;
            self.wifi = Some(CardWorderWifi::new(esp_wifi));
        }
        Ok(())
    }

    fn wifi(&mut self) -> anyhow::Result<&mut CardWorderWifi<'a>> {
        self.wifi.as_mut().ok_or_else(|| anyhow::anyhow!("WiFi not initialized"))
    }

    pub fn connect_wifi(&mut self, wifi_config: WifiConfig) -> anyhow::Result<()> {
        self.wifi()?.connect(wifi_config).map_err(|e| anyhow::anyhow!("Connect: {:?}", e))
    }

    pub fn stop_wifi(&mut self) -> anyhow::Result<()> {
        if let Some(mut wifi) = self.wifi.take() {
            wifi.stop().map_err(|e| anyhow::anyhow!("Stop: {:?}", e))?;
            drop(wifi); // drops EspWifi, frees ~50KB WiFi buffers
            // SAFETY: EspWifi deregistered the driver on drop, modem peripheral is free.
            // steal() is safe because no one else uses the modem after EspWifi is dropped.
            self.modem = Some(unsafe { esp_idf_hal::modem::Modem::steal() });
            log::info!("hal: WiFi dropped, modem recovered (~50KB freed)");
        }
        Ok(())
    }

    pub fn start_wifi(&mut self) -> anyhow::Result<()> {
        self.ensure_wifi()?;
        self.wifi()?.start().map_err(|e| anyhow::anyhow!("Start: {:?}", e))
    }

    pub fn scan_wifi(&mut self) -> anyhow::Result<Vec<esp_idf_svc::wifi::AccessPointInfo>> {
        self.wifi()?.scan().map_err(|e| anyhow::anyhow!("Scan: {:?}", e))
    }

    pub fn load_wifi_list(&mut self) -> anyhow::Result<crate::types::WifiConfigList> {
        use crate::types::WifiConfigList;
        let exists = self.sd.is_file_exists("wifilist.jsn")
            .map_err(|_| anyhow::anyhow!("Failed to check wifilist.jsn"))?;
        if !exists {
            return Ok(WifiConfigList::default());
        }
        let content = self.sd.read_file("wifilist.jsn")
            .map_err(|_| anyhow::anyhow!("Failed to read wifilist.jsn"))?;
        let list: WifiConfigList = serde_json::from_str(&content)?;
        Ok(list)
    }

    // ---- Two-file storage: FSRS.BIN (fixed records) + WORDS.BIN (postcard text) ----

    pub fn load_next_id(&mut self) -> anyhow::Result<u64> {
        use fsrs_core::{FsrsHeader, FSRS_HEADER_SIZE};
        let bytes = self.sd.read_file_bytes("FSRS.BIN").map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        if bytes.len() < FSRS_HEADER_SIZE {
            return Ok(1);
        }
        let header = FsrsHeader::from_bytes(bytes[..FSRS_HEADER_SIZE].try_into().unwrap())
            .ok_or_else(|| anyhow::anyhow!("Invalid FSRS header"))?;
        Ok(header.next_id)
    }

    /// Read entire FSRS.BIN in one I/O, parse records, filter due items into forward/reverse lists.
    pub fn load_due_items(&mut self) -> anyhow::Result<(Vec<fsrs_core::DueItem>, Vec<fsrs_core::DueItem>, u64)> {
        use fsrs_core::{FsrsHeader, FsrsRecord, DueItem, Direction, FSRS_HEADER_SIZE, FSRS_RECORD_SIZE};
        let exists = self.sd.is_file_exists("FSRS.BIN").map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        if !exists {
            return Ok((Vec::new(), Vec::new(), 1));
        }
        let bytes = self.sd.read_file_bytes("FSRS.BIN").map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        if bytes.len() < FSRS_HEADER_SIZE {
            return Ok((Vec::new(), Vec::new(), 1));
        }
        let header = FsrsHeader::from_bytes(bytes[..FSRS_HEADER_SIZE].try_into().unwrap())
            .ok_or_else(|| anyhow::anyhow!("Invalid FSRS header"))?;

        let record_data = &bytes[FSRS_HEADER_SIZE..];
        let record_count = record_data.len() / FSRS_RECORD_SIZE;
        let mut forward = Vec::new();
        let mut reverse = Vec::new();

        for slot in 0..record_count {
            let start = slot * FSRS_RECORD_SIZE;
            let end = start + FSRS_RECORD_SIZE;
            if end > record_data.len() { break; }
            let rec = FsrsRecord::from_bytes(record_data[start..end].try_into().unwrap());
            if rec.forward.is_due() {
                forward.push(DueItem { pair_id: rec.pair_id, direction: Direction::Forward, slot, word_offset: rec.word_offset, word_length: rec.word_length });
            }
            if rec.reverse.is_due() {
                reverse.push(DueItem { pair_id: rec.pair_id, direction: Direction::Reverse, slot, word_offset: rec.word_offset, word_length: rec.word_length });
            }
        }
        log::info!("load_due_items: {} records, fwd={} rev={} due", record_count, forward.len(), reverse.len());
        Ok((forward, reverse, header.next_id))
    }

    /// Read word text from WORDS.BIN at specific offset (single file open + seek).
    pub fn load_word_text(&mut self, word_offset: u32, word_length: u32) -> anyhow::Result<fsrs_core::WordText> {
        let mut buf = vec![0u8; word_length as usize];
        self.sd.read_at("WORDS.BIN", word_offset, &mut buf).map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        if buf.len() < 2 { return Err(anyhow::anyhow!("Word entry too short")); }
        let entry_len = u16::from_le_bytes([buf[0], buf[1]]) as usize;
        let data = &buf[2..2 + entry_len.min(buf.len() - 2)];
        let wt: fsrs_core::WordText = postcard::from_bytes(data).map_err(|e| anyhow::anyhow!("Postcard: {:?}", e))?;
        Ok(wt)
    }

    pub fn load_fsrs_record(&mut self, slot: usize) -> anyhow::Result<fsrs_core::FsrsRecord> {
        use fsrs_core::{FsrsRecord, FSRS_RECORD_SIZE};
        let mut buf = [0u8; FSRS_RECORD_SIZE];
        let offset = FsrsRecord::file_offset(slot);
        self.sd.read_at("FSRS.BIN", offset, &mut buf).map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        Ok(FsrsRecord::from_bytes(&buf))
    }

    pub fn save_fsrs_record(&mut self, slot: usize, record: &fsrs_core::FsrsRecord) -> anyhow::Result<()> {
        let offset = fsrs_core::FsrsRecord::file_offset(slot);
        let bytes = record.to_bytes();
        self.sd.write_at("FSRS.BIN", offset, &bytes).map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        Ok(())
    }

    /// Save a rating and load the next word+record in a single volume open.
    pub fn rate_and_load_next(
        &mut self,
        slot: usize,
        record: &fsrs_core::FsrsRecord,
        next_slot: usize,
        next_word_offset: u32,
        next_word_length: u32,
    ) -> anyhow::Result<(fsrs_core::WordText, fsrs_core::FsrsRecord)> {
        use fsrs_core::{FsrsRecord, FSRS_RECORD_SIZE};

        let write_offset = FsrsRecord::file_offset(slot);
        let next_rec_offset = FsrsRecord::file_offset(next_slot);

        let mut wt_buf = vec![0u8; next_word_length as usize];
        let mut rec_buf = [0u8; FSRS_RECORD_SIZE];

        self.sd.write_and_read_multi(
            "FSRS.BIN", write_offset, &record.to_bytes(),
            "WORDS.BIN", next_word_offset, &mut wt_buf,
            "FSRS.BIN", next_rec_offset, &mut rec_buf,
        ).map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;

        if wt_buf.len() < 2 { return Err(anyhow::anyhow!("Word entry too short")); }
        let entry_len = u16::from_le_bytes([wt_buf[0], wt_buf[1]]) as usize;
        let data = &wt_buf[2..2 + entry_len.min(wt_buf.len() - 2)];
        let wt: fsrs_core::WordText = postcard::from_bytes(data).map_err(|e| anyhow::anyhow!("Postcard: {:?}", e))?;

        Ok((wt, FsrsRecord::from_bytes(&rec_buf)))
    }

    pub fn add_pair(&mut self, en: &str, ru: &str) -> anyhow::Result<u64> {
        use fsrs_core::{FsrsHeader, FsrsRecord, BinaryDirState, WordText, FSRS_HEADER_SIZE};

        // Read or create header
        let exists = self.sd.is_file_exists("FSRS.BIN").map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        let next_id;
        if !exists {
            next_id = 1;
            let header = FsrsHeader::new(next_id);
            self.sd.write_file_bytes("FSRS.BIN", &header.to_bytes()).map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        } else {
            let mut hdr_buf = [0u8; FSRS_HEADER_SIZE];
            self.sd.read_at("FSRS.BIN", 0, &mut hdr_buf).map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
            let header = FsrsHeader::from_bytes(&hdr_buf).ok_or_else(|| anyhow::anyhow!("Invalid header"))?;
            next_id = header.next_id;
        }

        let pair_id = next_id;

        // Serialize word text
        let wt = WordText {
            en: en.to_string(), ru: ru.to_string(),
            examples: Vec::new(),
            created_at: fsrs_core::models::get_timestamp_iso(),
        };
        let wt_bytes = postcard::to_allocvec(&wt).map_err(|e| anyhow::anyhow!("Postcard: {:?}", e))?;
        let mut word_entry = Vec::with_capacity(2 + wt_bytes.len());
        word_entry.extend_from_slice(&(wt_bytes.len() as u16).to_le_bytes());
        word_entry.extend_from_slice(&wt_bytes);

        // Append word text to WORDS.BIN
        let word_offset = self.sd.append("WORDS.BIN", &word_entry).map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        let word_length = word_entry.len() as u32;

        // Create FSRS record
        let card = rs_fsrs::Card::new();
        let dir_state = BinaryDirState::from_card_and_review(&card, &None);
        let record = FsrsRecord {
            pair_id, word_offset, word_length,
            forward: dir_state.clone(), reverse: dir_state,
        };

        // Append FSRS record + update header
        self.sd.append("FSRS.BIN", &record.to_bytes()).map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        let new_header = FsrsHeader::new(pair_id + 1);
        self.sd.write_at("FSRS.BIN", 0, &new_header.to_bytes()).map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;

        log::info!("add_pair: id={}", pair_id);
        Ok(pair_id)
    }

    /// Compute stats from FSRS.BIN only (single read), then 3 small word text lookups.
    pub fn load_quick_stats(&mut self) -> anyhow::Result<QuickStats> {
        use fsrs_core::{FsrsRecord, FSRS_HEADER_SIZE, FSRS_RECORD_SIZE};

        let empty = || QuickStats { total_pairs: 0, forward: DirStats::default(), reverse: DirStats::default() };

        let fsrs_exists = self.sd.is_file_exists("FSRS.BIN").map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        if !fsrs_exists { return Ok(empty()); }

        let fsrs_bytes = self.sd.read_file_bytes("FSRS.BIN").map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        if fsrs_bytes.len() < FSRS_HEADER_SIZE { return Ok(empty()); }

        let record_data = &fsrs_bytes[FSRS_HEADER_SIZE..];
        let record_count = record_data.len() / FSRS_RECORD_SIZE;

        // Per-direction accumulators
        struct Acc { due: usize, new_c: usize, learning: usize, mastered: usize,
            reviews: i64, lapses: i64, diff_sum: f64, diff_count: usize,
            hardest: Option<(usize, i32)>, strongest: Option<(usize, f64)>, weakest: Option<(usize, f64)> }
        impl Acc { fn new() -> Self { Acc { due:0, new_c:0, learning:0, mastered:0,
            reviews:0, lapses:0, diff_sum:0.0, diff_count:0, hardest:None, strongest:None, weakest:None } }
            fn process(&mut self, d: &fsrs_core::BinaryDirState, slot: usize) {
                if d.is_due() { self.due += 1; }
                let is_new = d.state == 0 && d.reps == 0;
                if is_new { self.new_c += 1; }
                else if d.state == 2 && !d.is_due() { self.mastered += 1; }
                else { self.learning += 1; }
                self.reviews += d.reps as i64;
                self.lapses += d.lapses as i64;
                if d.reps > 0 { self.diff_sum += d.difficulty; self.diff_count += 1; }
                if d.lapses > 0 && (self.hardest.is_none() || d.lapses > self.hardest.unwrap().1) {
                    self.hardest = Some((slot, d.lapses)); }
                if d.reps > 0 {
                    if self.strongest.is_none() || d.stability > self.strongest.unwrap().1 { self.strongest = Some((slot, d.stability)); }
                    if self.weakest.is_none() || d.stability < self.weakest.unwrap().1 { self.weakest = Some((slot, d.stability)); }
                }
            }
        }

        let mut fwd = Acc::new();
        let mut rev = Acc::new();

        for slot in 0..record_count {
            let start = slot * FSRS_RECORD_SIZE;
            let end = start + FSRS_RECORD_SIZE;
            if end > record_data.len() { break; }
            let rec = FsrsRecord::from_bytes(record_data[start..end].try_into().unwrap());
            fwd.process(&rec.forward, slot);
            rev.process(&rec.reverse, slot);
        }

        // Collect notable records before dropping fsrs_bytes
        let collect_notable = |acc: &Acc| -> (Option<FsrsRecord>, Option<FsrsRecord>, Option<FsrsRecord>, i32, i64, i64) {
            let rec_at = |slot: usize| -> Option<FsrsRecord> {
                let start = slot * FSRS_RECORD_SIZE;
                Some(FsrsRecord::from_bytes(record_data[start..start+FSRS_RECORD_SIZE].try_into().ok()?))
            };
            let h_rec = acc.hardest.and_then(|(s, _)| rec_at(s));
            let s_rec = acc.strongest.and_then(|(s, _)| rec_at(s));
            let w_rec = acc.weakest.and_then(|(s, _)| rec_at(s));
            let h_lapses = acc.hardest.map(|(_, l)| l).unwrap_or(0);
            let s_days = s_rec.as_ref().map(|r| r.forward.scheduled_days.max(r.reverse.scheduled_days)).unwrap_or(0);
            let w_days = w_rec.as_ref().map(|r| r.forward.scheduled_days.min(r.reverse.scheduled_days)).unwrap_or(0);
            (h_rec, s_rec, w_rec, h_lapses, s_days, w_days)
        };

        let (fh, fs, fw, fhl, fsd, fwd_wd) = collect_notable(&fwd);
        let (rh, rs, rw, rhl, rsd, rev_wd) = collect_notable(&rev);

        drop(fsrs_bytes);

        // Load word labels (up to 6 seeks)
        let load_label = |rec: &Option<FsrsRecord>, sd: &mut CardputerSd<'_, Delay>| -> Option<String> {
            let r = rec.as_ref()?;
            let mut buf = vec![0u8; r.word_length as usize];
            sd.read_at("WORDS.BIN", r.word_offset, &mut buf).ok()?;
            if buf.len() < 2 { return None; }
            let el = u16::from_le_bytes([buf[0], buf[1]]) as usize;
            let wt: fsrs_core::WordText = postcard::from_bytes(&buf[2..2+el.min(buf.len()-2)]).ok()?;
            Some(format!("{} - {}", wt.en, wt.ru))
        };

        let build_dir = |acc: Acc, h_rec, s_rec, w_rec, hl, sd_val, wd, sd: &mut CardputerSd<'_, Delay>| -> DirStats {
            DirStats {
                due: acc.due, new_count: acc.new_c, learning: acc.learning, mastered: acc.mastered,
                total_reviews: acc.reviews, total_lapses: acc.lapses,
                avg_difficulty: if acc.diff_count > 0 { (acc.diff_sum / acc.diff_count as f64) as f32 } else { 0.0 },
                hardest_word: load_label(&h_rec, sd), hardest_lapses: hl,
                strongest_word: load_label(&s_rec, sd), strongest_days: sd_val,
                weakest_word: load_label(&w_rec, sd), weakest_days: wd,
            }
        };

        let forward = build_dir(fwd, fh, fs, fw, fhl, fsd, fwd_wd, self.sd);
        let reverse = build_dir(rev, rh, rs, rw, rhl, rsd, rev_wd, self.sd);

        log::info!("load_quick_stats: {} pairs, fwd_due={} rev_due={}", record_count, forward.due, reverse.due);

        Ok(QuickStats { total_pairs: record_count, forward, reverse })
    }

    pub fn migrate_pairs_if_needed(&mut self) -> anyhow::Result<bool> {
        use fsrs_core::{FsrsHeader, FsrsRecord, BinaryDirState, WordText, FSRS_HEADER_SIZE, FSRS_RECORD_SIZE};

        let fsrs_exists = self.sd.is_file_exists("FSRS.BIN").map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        if fsrs_exists { return Ok(false); }

        let pairs_exists = self.sd.is_file_exists("PAIRS.BIN").map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        if !pairs_exists { return Ok(false); }

        log::info!("migrate: converting PAIRS.BIN → FSRS.BIN + WORDS.BIN");
        let bytes = self.sd.read_file_bytes("PAIRS.BIN").map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        let old: fsrs_core::PairsFile = postcard::from_bytes(&bytes).map_err(|e| anyhow::anyhow!("Postcard: {:?}", e))?;

        // Build both files in memory, write once each
        let mut words_buf = Vec::new();
        let mut fsrs_buf = Vec::with_capacity(FSRS_HEADER_SIZE + old.pairs.len() * FSRS_RECORD_SIZE);

        let header = FsrsHeader::new(old.next_id);
        fsrs_buf.extend_from_slice(&header.to_bytes());

        for pair in &old.pairs {
            let wt = WordText {
                en: pair.en.clone(), ru: pair.ru.clone(),
                examples: pair.examples.clone(), created_at: pair.created_at.clone(),
            };
            let wt_bytes = postcard::to_allocvec(&wt).map_err(|e| anyhow::anyhow!("Postcard: {:?}", e))?;

            let word_offset = words_buf.len() as u32;
            words_buf.extend_from_slice(&(wt_bytes.len() as u16).to_le_bytes());
            words_buf.extend_from_slice(&wt_bytes);
            let word_length = (2 + wt_bytes.len()) as u32;

            let fwd = &pair.fsrs[0];
            let rev = &pair.fsrs[1];
            let record = FsrsRecord {
                pair_id: pair.id,
                word_offset, word_length,
                forward: BinaryDirState::from_card_and_review(&fwd.card, &fwd.last_review),
                reverse: BinaryDirState::from_card_and_review(&rev.card, &rev.last_review),
            };
            fsrs_buf.extend_from_slice(&record.to_bytes());
        }

        // Single write per file
        self.sd.write_file_bytes("FSRS.BIN", &fsrs_buf).map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;
        self.sd.write_file_bytes("WORDS.BIN", &words_buf).map_err(|e| anyhow::anyhow!("SD: {:?}", e))?;

        log::info!("migrate: done — {} pairs converted", old.pairs.len());
        Ok(true)
    }

    pub fn save_wifi_list(&mut self, list: &crate::types::WifiConfigList) -> anyhow::Result<()> {
        let content = serde_json::to_string(list)?;
        log::info!("save_wifi_list: writing {} bytes to wifilist.jsn", content.len());
        self.sd.write_file("wifilist.jsn", &content)
            .map_err(|e| {
                log::error!("save_wifi_list: SD write error: {:?}", e);
                anyhow::anyhow!("SD write error: {:?}", e)
            })?;
        Ok(())
    }
}

/// LiPo voltage (mV) to percentage (piecewise linear)
pub fn battery_mv_to_percent(mv: u32) -> u8 {
    if mv >= 4200 { return 100; }
    if mv <= 3500 { return 0; }
    const TABLE: [(u32, u8); 11] = [
        (4200, 100), (4060, 90), (3980, 80), (3920, 70), (3870, 60),
        (3820, 50), (3790, 40), (3770, 30), (3740, 20), (3680, 10), (3500, 0),
    ];
    for i in 0..TABLE.len() - 1 {
        let (v_hi, p_hi) = TABLE[i];
        let (v_lo, p_lo) = TABLE[i + 1];
        if mv >= v_lo {
            let range_v = (v_hi - v_lo) as u32;
            let range_p = (p_hi - p_lo) as u32;
            return (p_lo as u32 + (mv - v_lo) * range_p / range_v.max(1)) as u8;
        }
    }
    0
}
