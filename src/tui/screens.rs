use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
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
// Helpers — opencode-style prompt box
// ---------------------------------------------------------------------------

/// Short display names for the horizontal menu.
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

/// Render the opencode-style prompt box: left-only `┃` border in accent,
/// filled with `backgroundElement`, content rendered inside.
///
/// `box_h` must include room for the left border rows. Inner content is
/// placed at `x + 3, y + 1` with width `box_w - 5` and height `box_h - 2`.
/// Returns the area of the inner content region.
fn render_prompt_box(frame: &mut Frame, box_area: Rect) -> Rect {
    let w = box_area.width as usize;
    let h = box_area.height as usize;
    if w == 0 || h == 0 {
        return box_area;
    }

    // 1) Fill entire area with backgroundElement (each row = w spaces)
    let fill: String = " ".repeat(w);
    let bg_lines: Vec<Line> = (0..h)
        .map(|_| Line::from(Span::styled(fill.as_str(), theme::bg_element())))
        .collect();
    frame.render_widget(Paragraph::new(bg_lines), box_area);

    // 2) Left border — `┃` in accent colour, one per row
    let accent = Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD);
    let border_lines: Vec<Line> = (0..h)
        .map(|_| Line::from(Span::styled("┃", accent.clone())))
        .collect();
    frame.render_widget(
        Paragraph::new(border_lines),
        Rect {
            x: box_area.x,
            y: box_area.y,
            width: 1,
            height: box_area.height,
        },
    );

    // 3) Return inner area (padded: x+3 for left pad, y+1 for top pad)
    Rect {
        x: box_area.x + 3,
        y: box_area.y + 1,
        width: box_area.width.saturating_sub(5),
        height: box_area.height.saturating_sub(2),
    }
}

// ---------------------------------------------------------------------------
// Home — opencode header + prompt box with horizontal operation menu
// ---------------------------------------------------------------------------

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
        let box_w: u16 = 75.min(area.width.saturating_sub(4));
        // Inner: placeholder row + operations row = 2 content rows + 2 padding = 4
        let box_h: u16 = 4;
        let hint_h: u16 = 1;
        let tip_h: u16 = 1;

        let stack_h = header_h
            .saturating_add(1) // gap after header
            .saturating_add(box_h)
            .saturating_add(1) // gap after box
            .saturating_add(hint_h)
            .saturating_add(2) // gap before tip
            .saturating_add(tip_h);

        let start_y = if area.height > stack_h {
            area.y + (area.height - stack_h) / 2
        } else {
            area.y
        };
        let box_x = area.x + (area.width.saturating_sub(box_w)) / 2;
        let mut y = start_y;

        // ── Header (opencode-style PULP) ──
        let header_area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: header_h.min(area.height.saturating_sub(y - area.y)),
        };
        header::render(frame, header_area);
        y = y.saturating_add(header_h + 1);

        // ── Prompt box (opencode left-border + bg fill) ──
        if y + box_h <= area.y + area.height {
            let box_area = Rect {
                x: box_x,
                y,
                width: box_w,
                height: box_h,
            };
            let inner = render_prompt_box(frame, box_area);

            // Row 0: placeholder-like prompt
            if inner.height >= 1 {
                let prompt_line = Line::from(vec![
                    Span::styled("▌ ", Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
                    Span::styled(
                        "Ask anything… or choose an operation",
                        Style::new().fg(theme::SLATE),
                    ),
                ]);
                frame.render_widget(
                    Paragraph::new(prompt_line),
                    Rect {
                        x: inner.x,
                        y: inner.y,
                        width: inner.width,
                        height: 1,
                    },
                );
            }

            // Row 1: horizontal `·` menu
            if inner.height >= 2 {
                let selected = self.list.selected().copied();
                let mut spans: Vec<Span> = vec![];
                for (idx, op) in Operation::ALL.iter().enumerate() {
                    if idx > 0 {
                        spans.push(Span::styled(" · ", Style::new().fg(theme::SLATE)));
                    }
                    let title = short_title(op);
                    let style = if Some(*op) == selected {
                        Style::new()
                            .fg(theme::TITLE)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::new().fg(theme::SLATE)
                    };
                    spans.push(Span::styled(title, style));
                }
                let ops_line = Line::from(spans);
                frame.render_widget(
                    Paragraph::new(ops_line),
                    Rect {
                        x: inner.x,
                        y: inner.y + 1,
                        width: inner.width,
                        height: 1,
                    },
                );
            }

            y = y.saturating_add(box_h + 1);
        }

        // ── Hints (right-aligned) ──
        if y + hint_h <= area.y + area.height {
            let hint_text = "↑↓ select  enter run  q quit";
            let hint_w = hint_text.len() as u16;
            let hint_x = box_x
                .saturating_add(box_w)
                .saturating_sub(hint_w)
                .max(area.x);
            let hint_area = Rect {
                x: hint_x,
                y,
                width: hint_w.min(area.width.saturating_sub(hint_x.saturating_sub(area.x))),
                height: hint_h,
            };
            let hint_line = Line::from(vec![
                Span::styled("↑↓", Style::new().fg(theme::SLATE_300).add_modifier(Modifier::BOLD)),
                Span::styled(" select  ", Style::new().fg(theme::SLATE)),
                Span::styled("enter", Style::new().fg(theme::SLATE_300).add_modifier(Modifier::BOLD)),
                Span::styled(" run  ", Style::new().fg(theme::SLATE)),
                Span::styled("q", Style::new().fg(theme::SLATE_300).add_modifier(Modifier::BOLD)),
                Span::styled(" quit", Style::new().fg(theme::SLATE)),
            ]);
            frame.render_widget(
                Paragraph::new(hint_line).right_aligned(),
                hint_area,
            );
            y = y.saturating_add(hint_h + 2);
        }

        // ── Tip (centered, cyan bullet) ──
        if y + tip_h <= area.y + area.height {
            let tip_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: tip_h,
            };
            let tip = Line::from(vec![
                Span::styled("• ", Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
                Span::styled("Tip: ", Style::new().fg(theme::SLATE).add_modifier(Modifier::BOLD)),
                Span::styled(
                    "Drag and drop a PDF file or pass a path as an argument",
                    Style::new().fg(theme::SLATE),
                ),
            ]);
            frame.render_widget(Paragraph::new(tip).centered(), tip_area);
        }
    }
}

// ---------------------------------------------------------------------------
// Operation placeholder — same left-border box, centered.
// ---------------------------------------------------------------------------

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

        let box_w: u16 = 52.min(area.width.saturating_sub(4));
        let box_h: u16 = 7;
        let hint_h: u16 = 1;

        let stack_h = box_h.saturating_add(1).saturating_add(hint_h);
        let start_y = if area.height > stack_h {
            area.y + (area.height - stack_h) / 2
        } else {
            area.y
        };
        let box_x = area.x + (area.width.saturating_sub(box_w)) / 2;
        let mut y = start_y;

        // ── Breadcrumb ──
        let breadcrumb_area = Rect {
            x: box_x,
            y,
            width: box_w,
            height: 1,
        };
        header::breadcrumb_render(frame, breadcrumb_area, Some(self.operation.slug()));
        y += 2;

        // ── Prompt box (left-border + bg fill) ──
        if y + box_h <= area.y + area.height {
            let box_area = Rect {
                x: box_x,
                y,
                width: box_w,
                height: box_h,
            };
            let inner = render_prompt_box(frame, box_area);

            if inner.height >= 3 {
                // Row 0: operation title
                frame.render_widget(
                    Paragraph::new(Line::styled(
                        self.operation.title(),
                        Style::new().fg(theme::TITLE).add_modifier(Modifier::BOLD),
                    )),
                    Rect {
                        x: inner.x,
                        y: inner.y,
                        width: inner.width,
                        height: 1,
                    },
                );

                // Row 1: operation description
                frame.render_widget(
                    Paragraph::new(Line::styled(
                        self.operation.description(),
                        Style::new().fg(theme::SLATE),
                    )),
                    Rect {
                        x: inner.x,
                        y: inner.y + 1,
                        width: inner.width,
                        height: 1,
                    },
                );

                // Row 3: status
                self.status.render(
                    frame,
                    Rect {
                        x: inner.x,
                        y: inner.y + 3,
                        width: inner.width,
                        height: 1,
                    },
                );
            }

            y = y.saturating_add(box_h + 1);
        }

        // ── Hints ──
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
