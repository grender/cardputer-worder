// #![no_std] // can't cuz there is many format! macro

pub mod cardputer_hal;
pub mod ui;
pub mod logic;

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