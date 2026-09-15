use ratatui::style::{Color, Modifier, Style};

/// Dark slate / cyan aesthetic — OpenCode-inspired.
/// Keep this as the single source of truth for colour.
pub const SLATE: Color = Color::Rgb(100, 116, 139); // #64748B — hints, borders, muted
pub const SLATE_DIM: Color = Color::Rgb(71, 85, 105); // #475569 — subtle variant
pub const SLATE_300: Color = Color::Rgb(203, 213, 225); // #CBD5E1 — requested header color
pub const ACCENT: Color = Color::Rgb(56, 189, 248); // #38BDF8 — cyan highlight
pub const ACCENT_DIM: Color = Color::Rgb(14, 165, 233); // #0EA5E9
pub const TITLE: Color = Color::Rgb(248, 250, 252); // #F8FAFC — crisp white
pub const RED: Color = Color::Rgb(248, 113, 113); // soft red for errors

// Legacy aliases so existing code keeps compiling.
pub const MUTED: Color = SLATE;

/// Accent (cyan) — highlights, selection marker, tip bullet.
pub fn accent() -> Style {
    Style::new().fg(ACCENT)
}

pub fn accent_bold() -> Style {
    accent().add_modifier(Modifier::BOLD)
}

/// Muted slate — hints, descriptions, borders.
pub fn muted() -> Style {
    Style::new().fg(SLATE)
}

pub fn muted_bold() -> Style {
    muted().add_modifier(Modifier::BOLD)
}

pub fn muted_italic() -> Style {
    muted().add_modifier(Modifier::ITALIC)
}

/// Crisp white — titles and active/selected item.
pub fn title() -> Style {
    Style::new().fg(TITLE).add_modifier(Modifier::BOLD)
}

/// Requested header style — Slate 300 + BOLD (`Style::default().fg(Rgb(203,213,225)).add_modifier(BOLD)`).
pub fn header() -> Style {
    Style::default()
        .fg(SLATE_300)
        .add_modifier(Modifier::BOLD)
}

pub fn primary_bold() -> Style {
    title()
}

/// Border style for the centered card.
pub fn border() -> Style {
    Style::new().fg(SLATE)
}

pub fn border_focused() -> Style {
    Style::new().fg(SLATE)
}

/// Style for the currently selected list row: white bold.
pub fn selected() -> Style {
    Style::new().fg(TITLE).add_modifier(Modifier::BOLD)
}

/// Highlight marker (▸ / ›) rendered in cyan so selection pops.
pub fn highlight_marker() -> Style {
    Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn error() -> Style {
    Style::new().fg(RED)
}
