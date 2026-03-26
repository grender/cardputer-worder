use std::sync::mpsc::Sender;

use crate::cardputer_hal::cardputer_hal::CardputerHal;
use crate::types::{Command, Msg, SharedState};
use crate::ui::cardworder_ui::CardworderUi;

/// Implemented by each screen's snapshot. Core 0 calls `draw()` on this.
/// Must be `Send` because snapshots cross from Core 1 to Core 0 via channel.
pub trait Renderable: Send {
    /// Draw the snapshot onto the framebuffer. Called on Core 0.
    fn draw(&self, ui: &mut CardworderUi);

    /// Whether the top bar (clock, input state) should be drawn.
    fn needs_top_line(&self) -> bool {
        true
    }
}

/// Implemented on the Core 1 side. One screen is active at a time.
pub trait Screen: Send {
    /// Called once when this screen becomes active (Core 1).
    /// `state_tx` allows sending intermediate snapshots during long operations.
    fn on_mount(
        &mut self,
        _hal: &mut CardputerHal<'_>,
        _shared: &SharedState,
        _state_tx: &Sender<Box<dyn Renderable>>,
    ) -> Command {
        Command::None
    }

    /// Core 1: process a message, mutate self, return a command.
    fn handle_msg(
        &mut self,
        msg: Msg,
        hal: &mut CardputerHal<'_>,
        shared: &SharedState,
    ) -> Command;

    /// Core 1: produce a snapshot for Core 0 to draw.
    fn snapshot(&self) -> Box<dyn Renderable>;
}
