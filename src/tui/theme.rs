use ratatui::style::{Color, Modifier, Style};

/// Dark slate / cyan aesthetic — OpenCode-inspired.
/// Keep this as the single source of truth for colour.
pub const SLATE: Color = Color::Rgb(100, 116, 139); // #64748B — hints, borders, muted
pub const SLATE_300: Color = Color::Rgb(203, 213, 225); // #CBD5E1 — header, bold hints
pub const ACCENT: Color = Color::Rgb(56, 189, 248); // #38BDF8 — cyan highlight
pub const TITLE: Color = Color::Rgb(248, 250, 252); // #F8FAFC — crisp white
pub const RED: Color = Color::Rgb(248, 113, 113); // soft red for errors

/// Assumed page background used to blend the header's translucent shadow
/// cells. Matches the dark slate backdrop the TUI draws over.
pub const PAGE_BG: Color = Color::Rgb(11, 15, 25);

/// Opencode's `theme.backgroundElement` — a subtle lighter-than-page fill
/// behind prompt boxes and interactive areas. Roughly 3% white overlay on
/// PAGE_BG; produces a barely-visible brightening of the region.
pub const BG_ELEMENT: Color = Color::Rgb(16, 21, 32);

// ---------------------------------------------------------------------------
// Tint / shadow — used by the header mark system
// ---------------------------------------------------------------------------

/// OpenCode's `tint` — blend `overlay` toward `base` by `alpha` (0..=1).
pub fn tint(base: Color, overlay: Color, alpha: f32) -> Color {
    let mix = |b: u8, o: u8| (b as f32 + (o as f32 - b as f32) * alpha).round() as u8;
    match (base, overlay) {
        (Color::Rgb(br, bg, bb), Color::Rgb(or, og, ob)) => {
            Color::Rgb(mix(br, or), mix(bg, og), mix(bb, ob))
        }
        _ => overlay,
    }
}

/// Shadow tone for header glyph cells, mirroring `Logo()`:
/// `shadow = tint(theme.background, fg, 0.25)`.
pub fn shadow(fg: Color) -> Color {
    tint(PAGE_BG, fg, 0.25)
}

// ---------------------------------------------------------------------------
// Convenience styles — used by header, screens, status, footer
// ---------------------------------------------------------------------------

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

/// Requested header style — Slate 300 + BOLD.
pub fn header() -> Style {
    Style::default()
        .fg(SLATE_300)
        .add_modifier(Modifier::BOLD)
}

pub fn error() -> Style {
    Style::new().fg(RED)
}

/// Style for the currently selected row: white bold.
pub fn selected() -> Style {
    Style::new().fg(TITLE).add_modifier(Modifier::BOLD)
}

/// Highlight marker (▸ / ›) rendered in cyan so selection pops.
pub fn highlight_marker() -> Style {
    Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)
}

/// Background element style — subtle fill behind the prompt box.
pub fn bg_element() -> Style {
    Style::new().bg(BG_ELEMENT)
}
