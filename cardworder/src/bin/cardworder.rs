use cardworder::cardputer_hal::cardputer_hal::CardputerHal;
use cardworder::logic::view_manager::{ViewManager};
use cardworder::logic::views::main_menu::MainMenuView;
use cardworder::logic::views::start::StartView;
use cardworder::ui::cardworder_ui::CardworderUi;
use cardworder::ResultExt;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;


fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Start the app");

    log::info!("boot step 1: Peripherals::take");
    let peripherals = Peripherals::take().unwrap_or_log("error get peripherals");
    log::info!("boot step 2: EspSystemEventLoop::take");
    let sysloop = EspSystemEventLoop::take().unwrap_or_log("error init event loop");
    log::info!("boot step 3: CardputerHal::new");
    let mut hal = CardputerHal::new(peripherals, sysloop /*.clone()*/);
    log::info!("boot step 4: hal.take_screen");
    let screen = hal.take_screen();
    log::info!("boot step 5: CardworderUi::build");
    let ui = CardworderUi::build(screen);
    log::info!("boot step 6: ViewManager::new (MainMenuView)");
    let mut view_manager = ViewManager::new(hal, ui, Box::new(MainMenuView::default()));
    log::info!("boot: init done, entering main loop");

    loop {
        FreeRtos::delay_ms(1);
        view_manager.loop_logic();
    }
}
