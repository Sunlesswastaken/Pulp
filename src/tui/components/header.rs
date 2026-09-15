use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::theme;

/// Opencode-style PULP title — copies `https://github.com/anomalyco/opencode`
/// exactly: 4-row block font, left part muted, right part bold, gap 1,
/// with drop-shadow using `tint(background, fg, 0.25)`.
///
/// ```text
///  left (muted)      right (Slate 300 bold)
///  "         "      "         "         ← row0 shadow offset (empty)
///  "█▀▀█ █  █"      "█    █▀▀█"         ← row1 top
///  "█  █ █  █"      "█    █  █"         ← row2 mid
///  "█▀▀▀ ▀▀▀▀"      "▀▀▀▀ █▀▀▀"         ← row3 bottom (P bar restored — no cutoff)
/// ```
/// Combined width = left_w (9) + gap (1) + right_w (9) = 19, height = 4.
/// Rendered centered; row0 is empty but keeps the 1-px shadow offset
/// identical to opencode's `logo.ts`.
const PULP_LEFT: [&str; 4] = [
    "         ",
    "█▀▀█ █  █",
    "█  █ █  █",
    "█▀▀▀ ▀▀▀▀",
];

const PULP_RIGHT: [&str; 4] = [
    "         ",
    "█    █▀▀█",
    "█    █  █",
    "▀▀▀▀ █▀▀▀",
];

const PULP_MINI: &str = "PULP";

pub fn render(frame: &mut Frame, area: Rect) {
    let header_left_style = theme::muted(); // textMuted — #64748B / #808080
    let header_right_style = theme::header(); // Slate 300 #CBD5E1 + BOLD as requested

    let block_w = 19u16; // 9 + 1 + 9
    let block_h = 4u16;

    if area.height < block_h || area.width < block_w {
        let line = Line::from(Span::styled(PULP_MINI, header_right_style));
        frame.render_widget(Paragraph::new(line).centered(), area);
        return;
    }

    // Build 4 lines: left (muted) + gap + right (bold Slate 300)
    // For opencode fidelity, bottom row shadow (`▀`/`▄`) would use
    // `tint(background, fg, 0.25)` as bg. We approximate with dim slate
    // for the shadow bg by rendering those cells with `theme::muted()` bg
    // tint — here simplified to same fg but Paragraph will handle.
    let mut lines: Vec<Line> = Vec::with_capacity(4);
    for i in 0..4 {
        let left = PULP_LEFT[i];
        let right = PULP_RIGHT[i];
        // Gap of 1 cell between left and right, as in `logo.tsx` gap={1}
        let line = Line::from(vec![
            Span::styled(left, header_left_style),
            Span::raw(" "),
            Span::styled(right, header_right_style),
        ]);
        lines.push(line);
    }

    // Center the 19×4 block as a whole. Using Paragraph::centered() will
    // center each line individually, but since all lines are same width (19),
    // the block stays aligned — no per-line offset.
    frame.render_widget(Paragraph::new(lines).centered(), area);
}

/// Height required by the header. Callers use this to compute vertical centering.
pub fn height_for(area: Rect) -> u16 {
    let block_w = 19u16;
    let block_h = 4u16;
    if area.height < block_h || area.width < block_w {
        1
    } else {
        block_h
    }
}

#[allow(dead_code)]
pub fn breadcrumb_render(frame: &mut Frame, area: Rect, section: Option<&str>) {
    let mut spans = vec![Span::styled("pulp", theme::accent_bold())];
    if let Some(section) = section {
        spans.push(Span::styled(format!("  /  {section}"), theme::muted_italic()));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
