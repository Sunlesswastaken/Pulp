use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Margin, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::core::operation::Operation;

use super::components::{header, nav_list::NavList, status::Status};
use super::theme;

/// A screen-level action that affects the app shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    Quit,
    Open(Operation),
    Back,
}

/// The screen currently shown.
#[derive(Debug)]
pub enum Screen {
    Home,
    Operation(OperationScreen),
}

// ---------------------------------------------------------------------------
// Home — opencode header + horizontal action menu
// ---------------------------------------------------------------------------

/// Short display names for the horizontal menu — matches the requested
/// `Compress · Merge · Split · Remove · Extract · Password` (without
/// the ` pages` suffix) to keep the `·` row under ~55 cols.
fn short_title(op: &Operation) -> &'static str {
    match op {
        Operation::Compress => "Compress",
        Operation::Merge => "Merge",
        Operation::Split => "Split",
        Operation::Remove => "Remove",
        Operation::Extract => "Extract",
        Operation::Password => "Password",
        Operation::Info => "Info",
    }
}

/// Home: pick an operation via the horizontal `·` menu.
#[derive(Debug)]
pub struct HomeScreen {
    list: NavList<Operation>,
}

impl HomeScreen {
    pub fn new() -> Self {
        Self {
            list: NavList::new("PDF toolkit", Operation::ALL),
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> Option<Transition> {
        match key.code {
            KeyCode::Down
            | KeyCode::Right
            | KeyCode::Char('j')
            | KeyCode::Char('l') => self.list.select_next(),
            KeyCode::Up | KeyCode::Left | KeyCode::Char('k') | KeyCode::Char('h') => {
                self.list.select_previous()
            }
            KeyCode::Enter => return self.list.selected().map(|op| Transition::Open(*op)),
            KeyCode::Esc | KeyCode::Char('q') => return Some(Transition::Quit),
            _ => {}
        }
        None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let header_h = header::height_for(area);
        // New menu card: 2 content rows + borders = 4 (was 11 for vertical list)
        // Width must fit the `·` row (~54 for 6 items, ~61 with Info)
        let ops_line = self.operations_line_len() as u16;
        let prompt_len: u16 = "▌ Select action.. (or search)".len() as u16;
        let inner_needed = ops_line.max(prompt_len).saturating_add(4); // 2 left + 2 right padding
        let card_width: u16 = (inner_needed + 2).min(area.width.saturating_sub(4)).max(40);
        let card_height: u16 = 4; // 1 top border + 2 rows + 1 bottom border
        let hint_h: u16 = 1;
        let tip_h: u16 = 1;

        let stack_h = header_h
            .saturating_add(1)
            .saturating_add(card_height)
            .saturating_add(1) // gap to hints
            .saturating_add(hint_h)
            .saturating_add(2)
            .saturating_add(tip_h);

        let start_y = if area.height > stack_h {
            area.y + (area.height - stack_h) / 2
        } else {
            area.y
        };
        let card_x = area.x + (area.width.saturating_sub(card_width)) / 2;
        let mut y = start_y;

        // ── Header (opencode-style PULP, centered) ──
        let header_area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: header_h.min(area.height.saturating_sub(y - area.y)),
        };
        header::render(frame, header_area);
        y = y.saturating_add(header_h).saturating_add(1);

        // ── Card (centered, rounded borders, 2 rows inside) ──
        if y + card_height <= area.y + area.height {
            let card_area = Rect {
                x: card_x,
                y,
                width: card_width,
                height: card_height,
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme::border());
            frame.render_widget(block, card_area);

            let inner = card_area.inner(Margin {
                vertical: 1,
                horizontal: 2,
            });
            if inner.width > 0 && inner.height >= 2 {
                // Row 0: prompt `▌ Select action.. (or search)` — ▌ in cyan, rest muted italic
                let prompt_area = Rect {
                    x: inner.x,
                    y: inner.y,
                    width: inner.width,
                    height: 1,
                };
                let prompt_line = Line::from(vec![
                    Span::styled("▌ ", theme::accent_bold()),
                    Span::styled("Select action.. (or search)", theme::muted_italic()),
                ]);
                frame.render_widget(Paragraph::new(prompt_line), prompt_area);

                // Row 1: horizontal `·` menu — `Compress · Merge · …`
                let ops_area = Rect {
                    x: inner.x,
                    y: inner.y + 1,
                    width: inner.width,
                    height: 1,
                };
                // Indent 1-2 cols like the spec's `"   Compress · …"`
                let indent = "  ";
                let mut spans: Vec<Span> = vec![Span::raw(indent)];
                let selected = self.list.selected().copied();
                for (idx, op) in Operation::ALL.iter().enumerate() {
                    if idx > 0 {
                        spans.push(Span::styled(" · ", theme::muted()));
                    }
                    let title = short_title(op);
                    let is_selected = Some(*op) == selected;
                    let style = if is_selected {
                        theme::selected()
                    } else {
                        theme::muted()
                    };
                    spans.push(Span::styled(title.to_string(), style));
                }
                // If inner is narrower than ops line, truncate is handled by Paragraph
                frame.render_widget(Paragraph::new(Line::from(spans)), ops_area);
            }
            y = y.saturating_add(card_height).saturating_add(1);
        }

        // ── Hints — right-aligned under the card (as in the requested ascii)
        if y + hint_h <= area.y + area.height {
            // Align hints' right edge with the card's right edge, like
            // `                                         ↑↓ select  enter run`
            let hint_text = "↑↓ select  enter run";
            let hint_width = hint_text.len() as u16;
            let hint_x = card_x
                .saturating_add(card_width)
                .saturating_sub(hint_width);
            let hint_area = Rect {
                x: hint_x,
                y,
                width: hint_width.min(area.width.saturating_sub(hint_x - area.x)),
                height: hint_h,
            };
            // Render as muted with keys in bold, right-aligned
            let hint_line = Line::from(vec![
                Span::styled("↑↓", theme::muted_bold()),
                Span::styled(" select  ", theme::muted()),
                Span::styled("enter", theme::muted_bold()),
                Span::styled(" run", theme::muted()),
            ]);
            frame.render_widget(Paragraph::new(hint_line).right_aligned(), hint_area);
            y = y.saturating_add(hint_h).saturating_add(2);
        }

        // ── Tip line (centered, cyan bullet) ──
        if y + tip_h <= area.y + area.height {
            let tip_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: tip_h,
            };
            let tip = Line::from(vec![
                Span::styled("• ", theme::accent_bold()),
                Span::styled("Tip: ", theme::muted_bold()),
                Span::styled(
                    "Drag and drop a PDF file or pass a path as an argument",
                    theme::muted(),
                ),
            ]);
            frame.render_widget(Paragraph::new(tip).centered(), tip_area);
        }
    }

    fn operations_line_len(&self) -> usize {
        // "Compress · Merge · Split · Remove · Extract · Password · Info"
        // with " · " separators, plus indent
        let titles: Vec<&str> = Operation::ALL.iter().map(|op| short_title(op)).collect();
        let sep = " · ".len();
        let mut len = 2; // indent "  "
        for (i, t) in titles.iter().enumerate() {
            if i > 0 {
                len += sep;
            }
            len += t.len();
        }
        len
    }
}

// ---------------------------------------------------------------------------
// Operation placeholder — also centered, same card chrome.
// ---------------------------------------------------------------------------

/// Placeholder screen for an operation. Real operation screens replace
/// this in later phases; for now it sets up the breadcrumb, status and
/// navigation pattern every operation will follow.
#[derive(Debug)]
pub struct OperationScreen {
    pub operation: Operation,
    status: Status,
}

impl OperationScreen {
    pub fn new(operation: Operation) -> Self {
        let status = Status::info(format!(
            "{} is not implemented yet — it arrives in a later phase.",
            operation.title()
        ));
        Self { operation, status }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> Option<Transition> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Some(Transition::Back),
            _ => None,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let card_width: u16 = 52.min(area.width.saturating_sub(4));
        let card_height: u16 = 9.min(area.height);
        let header_h = 1u16;
        let hint_h: u16 = 1;

        let stack_h = header_h
            .saturating_add(1)
            .saturating_add(card_height)
            .saturating_add(1)
            .saturating_add(hint_h);

        let start_y = if area.height > stack_h {
            area.y + (area.height - stack_h) / 2
        } else {
            area.y
        };

        let card_x = area.x + (area.width.saturating_sub(card_width)) / 2;
        let mut y = start_y;

        let header_area = Rect {
            x: card_x,
            y,
            width: card_width,
            height: header_h,
        };
        header::breadcrumb_render(frame, header_area, Some(self.operation.slug()));
        y = y.saturating_add(header_h).saturating_add(1);

        if y + card_height <= area.y + area.height {
            let card_area = Rect {
                x: card_x,
                y,
                width: card_width,
                height: card_height,
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme::border());
            frame.render_widget(block, card_area);

            let inner = card_area.inner(Margin {
                vertical: 1,
                horizontal: 2,
            });

            if inner.height >= 4 {
                let title_area = Rect {
                    x: inner.x,
                    y: inner.y,
                    width: inner.width,
                    height: 1,
                };
                let desc_area = Rect {
                    x: inner.x,
                    y: inner.y + 1,
                    width: inner.width,
                    height: 1,
                };
                let status_area = Rect {
                    x: inner.x,
                    y: inner.y + 3,
                    width: inner.width,
                    height: 1,
                };
                frame.render_widget(
                    Paragraph::new(Line::styled(
                        self.operation.title(),
                        theme::primary_bold(),
                    )),
                    title_area,
                );
                frame.render_widget(
                    Paragraph::new(Line::styled(
                        self.operation.description(),
                        theme::muted_italic(),
                    )),
                    desc_area,
                );
                self.status.render(frame, status_area);
            }
            y = y.saturating_add(card_height).saturating_add(1);
        }

        if y + hint_h <= area.y + area.height {
            let hint_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: hint_h,
            };
            let hints = super::components::footer::HintBar::new()
                .hint("esc", "back")
                .hint("q", "quit");
            hints.render(frame, hint_area);
        }
    }
}
