use cardworder::cardputer_hal::cardputer_hal::CardputerHal;
use cardworder::logic::view_manager::{ViewManager};
use cardworder::logic::views::main_menu::MainMenuView;
use cardworder::logic::views::start::StartView;
use cardworder::ui::cardworder_ui::CardworderUi;
use cardworder::ResultExt;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::Peripherals;


fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Start the app");

    let peripherals = Peripherals::take().unwrap_or_log("error get peripherals");
    let sysloop = EspSystemEventLoop::take().unwrap_or_log("error init event loop");
    let mut hal = CardputerHal::new(peripherals, sysloop.clone());

    let screen = hal.take_screen();
    let ui = CardworderUi::build(screen);

    let mut view_manager = ViewManager::new(hal, ui, Box::new(MainMenuView::default()));

    loop {
        view_manager.loop_logic();
    }
}
