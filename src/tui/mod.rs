//! The interactive terminal UI.
//!
//! The look is deliberately restrained: default terminal background,
//! a single accent for selection, muted styling for secondary text, and
//! no borders around everything.

mod app;
mod components;
mod screens;
mod theme;

pub fn run(theme: Option<&str>) -> crate::error::Result<()> {
    crate::tui::theme::init(theme);
    app::run()
}
