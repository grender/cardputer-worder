// #![no_std] // can't cuz there is many format! macro

pub mod cardputer_hal;
pub mod core0_dispatch;
pub mod esp_util;
pub mod ui;
pub mod types;
pub mod screen;
pub mod runtime;
pub mod screens;

pub trait ResultExt<R, E> {
    fn unwrap_or_log(self, message: &str) -> R;
}

impl<R, E: core::fmt::Debug> ResultExt<R, E> for Result<R, E> {
    fn unwrap_or_log(self, message: &str) -> R {
        match self {
            Ok(t) => t,
            Err(e) => {
                log::error!("error: {} {:?}", message, e);
                loop {}
            }
        }
    }
}
