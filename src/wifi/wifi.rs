use esp_idf_svc::wifi::EspWifi;

pub struct CardWorderWifi<'a> {
    driver: &'a EspWifi<'a>
}