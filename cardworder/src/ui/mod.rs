// Re-export platform-agnostic UI modules from core.
pub use cardworder_core::ui::{cardworder_ui, elements, fonts, framebuffer, render};

// ESP-specific splash screen (uses CardputerKeyboard + FreeRtos).
pub mod splash;
