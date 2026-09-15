use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::tui::theme;

/// Opencode-style PULP title — ports `https://github.com/anomalyco/opencode`
/// `packages/tui/src/log.ts` + `component/logo.tsx` faithfully:
/// 4-row grid, left layer muted, right layer Slate 300 BOLD, gap 1, and the
/// four "mark" characters that fake translucency / inner shadow:
///
/// ```text
/// `_`  → " "  with `tint(background, fg, 0.25)` background  (inner shadow fill)
/// `^`  → ▀    fg on shadow background                       (lit top bevel)
/// `~`  → ▀    entirely shadow colour                        (ghost top)
/// `,`  → ▄    entirely shadow colour                        (ghost bottom)
/// ```
///
/// Left (muted "PU") and bright (right Slate 300 bold "LP") halves laid side
/// by side with gap 1, exactly like `logo.tsx`. Marks copied one-to-one from
/// opencode's conventions — hollow faces get the `_` inner-shadow fill:
///
/// ```text
///  left (muted "PU")    right (Slate 300 bold "LP")
///  "         "         "         "     ← row0 empty (shadow offset)
///  "█▀▀█ █  █"         "█    █▀▀█"     ← row1 top bars
///  "█__█ █__█"         "█    █__█"     ← row2 `_` = inner shadow (bowl/cup)
///  "█▀▀▀ ▀▀▀▀"         "▀▀▀▀ █▀▀▀"     ← row3 bottom bars
/// ```
const PULP_LEFT: [&str; 4] = ["         ", "█▀▀█ █  █", "█__█ █__█", "█▀▀▀ ▀▀▀▀"];

const PULP_RIGHT: [&str; 4] = ["         ", "█    █▀▀█", "█    █__█", "▀▀▀▀ █▀▀▀"];

const PULP_MINI: &str = "PULP";

/// Execute-step for one 9-col layer, mirroring `Logo.renderLine`:
/// `_` → shadow-bg space, `^` → ▀ on shadow-bg, `~`/`,` → plain shadow glyph.
fn render_line(line: &str, fg: Style, shadow: Color) -> Vec<Span<'static>> {
    line.chars()
        .map(|c| match c {
            '_' => Span::styled(" ", fg.bg(shadow)),
            '^' => Span::styled("▀", fg.bg(shadow)),
            '~' => Span::styled("▀", Style::new().fg(shadow)),
            ',' => Span::styled("▄", Style::new().fg(shadow)),
            other => Span::styled(other.to_string(), fg),
        })
        .collect()
}

pub fn render(frame: &mut Frame, area: Rect) {
    let header_left_style = theme::muted(); // textMuted — active theme
    let header_right_style = theme::header(); // text + BOLD — active theme
    let shadow_left = theme::shadow(theme::text_muted());
    let shadow_right = theme::shadow(theme::header_color());

    let block_w = 19u16; // 9 + 1 + 9
    let block_h = 4u16;

    if area.height < block_h || area.width < block_w {
        let line = Line::from(Span::styled(PULP_MINI, header_right_style));
        frame.render_widget(Paragraph::new(line).centered(), area);
        return;
    }

    let mut lines: Vec<Line> = Vec::with_capacity(4);
    for i in 0..4 {
        let left = render_line(PULP_LEFT[i], header_left_style, shadow_left);
        let right = render_line(PULP_RIGHT[i], header_right_style, shadow_right);
        // left (9) + gap (1) + right (9) = 19, as `logo.tsx` gap={1}.
        let mut spans = left;
        spans.push(Span::raw(" "));
        spans.extend(right);
        lines.push(Line::from(spans));
    }

    // All lines are the same width (19), so Paragraph::centered() keeps the
    // block aligned as a whole.
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
        spans.push(Span::styled(
            format!("  /  {section}"),
            theme::muted_italic(),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
