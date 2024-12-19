use embedded_graphics::{
    framebuffer,
    framebuffer::{Framebuffer, buffer_size},
    pixelcolor::{Rgb565, raw::LittleEndian},
    prelude::*,
    primitives::PrimitiveStyle,
};

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Start the app");


    ///
    let mut fb = Framebuffer::<Rgb565, _, LittleEndian, 320, 240, {buffer_size::<Rgb565>(320, 240)}>::new();
    ///

    loop {
        fb.bounding_box()
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 1))
        .draw(&mut fb)
        .unwrap();
    }
}