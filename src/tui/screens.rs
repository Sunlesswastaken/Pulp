use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::core::operation::Operation;

use super::components::{header, status::Status};
use super::theme;

/// A screen-level action that affects the app shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    Quit,
    Open(Operation),
    PickTheme,
    Back,
}

/// The screen currently shown.
#[derive(Debug)]
pub enum Screen {
    Home,
    Operation(OperationScreen),
    ThemePicker(ThemePickerScreen),
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
/// Mirrors `prompt/index.tsx`: the accent strip is the outer box's left
/// border on the page background, and the `backgroundElement` panel starts
/// one column in (`paddingLeft=2`), so the strip reads as detached from the
/// panel rather than glued to it.
///
/// Inner content is placed at `x + 3, y + 1` with width `box_w - 5` and
/// height `box_h - 2`. Returns the area of the inner content region.
fn render_prompt_box(frame: &mut Frame, box_area: Rect) -> Rect {
    let w = box_area.width as usize;
    let h = box_area.height as usize;
    if w == 0 || h == 0 {
        return box_area;
    }

    if w > 1 {
        // 1) Fill the panel (cols x+1..) with backgroundElement — the accent
        //    strip column stays on the page background, detaching the strip.
        let fill: String = " ".repeat(w - 1);
        let fill_area = Rect {
            x: box_area.x + 1,
            y: box_area.y,
            width: box_area.width - 1,
            height: box_area.height,
        };
        let bg_lines: Vec<Line> = (0..h)
            .map(|_| Line::from(Span::styled(fill.as_str(), theme::bg_element())))
            .collect();
        frame.render_widget(Paragraph::new(bg_lines), fill_area);
    }

    // 2) Left border — `┃` in accent colour, one per row.
    // Opencode: tint(theme.border, highlight(), alpha) where highlight =
    // the agent accent; pulp has no agents, so theme.accent directly.
    let border_style = Style::new().fg(theme::accent_color());
    let border_lines: Vec<Line> = (0..h)
        .map(|_| Line::from(Span::styled("┃", border_style)))
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

    // 3) Return inner area (x+1 panel start + paddingLeft=2 + paddingTop=1)
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
    /// Text typed into the prompt box. Empty = show placeholder.
    query: String,
    /// Character offset (byte index) of the caret within `query`.
    caret: usize,
    /// Operations matching `query` (all when query is empty).
    matches: Vec<Operation>,
    /// Index of the highlighted operation within `matches`.
    selected: usize,
}

impl HomeScreen {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            caret: 0,
            matches: Operation::ALL.to_vec(),
            selected: 0,
        }
    }

    /// Case-insensitive substring match against title, slug and short name.
    fn matches_query(query: &str, op: &Operation) -> bool {
        let q = query.to_lowercase();
        op.title().to_lowercase().contains(&q)
            || op.slug().contains(&q)
            || short_title(op).to_lowercase().contains(&q)
    }

    /// Recomputed the visible operation list from the current query and
    /// clamps the selection so it always stays valid.
    fn apply_query(&mut self) {
        if self.query.is_empty() {
            self.matches = Operation::ALL.to_vec();
        } else {
            self.matches = Operation::ALL
                .iter()
                .copied()
                .filter(|op| Self::matches_query(&self.query, op))
                .collect();
        }
        if self.selected >= self.matches.len() {
            self.selected = self.matches.len().saturating_sub(1);
        }
        if self.caret > self.query.len() {
            self.caret = self.query.len();
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> Option<Transition> {
        use ratatui::crossterm::event::KeyModifiers;

        // Ctrl+T opens the theme picker before any text editing kicks in.
        if key.code == KeyCode::Char('t') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Some(Transition::PickTheme);
        }

        match key.code {
            // ── text editing ──
            KeyCode::Char(c) if !c.is_control() => {
                self.query.insert(self.caret, c);
                let shift = c.len_utf8();
                self.caret = (self.caret + shift).min(self.query.len());
                self.apply_query();
            }
            KeyCode::Backspace => {
                if self.caret > 0 {
                    let idx = self.query[..self.caret]
                        .char_indices()
                        .next_back()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.query.remove(idx);
                    self.caret = idx;
                    self.apply_query();
                }
            }
            KeyCode::Delete => {
                if self.caret < self.query.len() {
                    self.query.remove(self.caret);
                    self.apply_query();
                }
            }
            KeyCode::Left => {
                if self.caret > 0 {
                    let idx = self.query[..self.caret]
                        .char_indices()
                        .next_back()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.caret = idx;
                }
            }
            KeyCode::Right => {
                if self.caret < self.query.len() {
                    self.caret += self.query[self.caret..]
                        .chars()
                        .next()
                        .map(|c| c.len_utf8())
                        .unwrap_or(0);
                }
            }
            KeyCode::Home => self.caret = 0,
            KeyCode::End => self.caret = self.query.len(),
            // ── navigation ──
            KeyCode::Up => {
                if !self.matches.is_empty() {
                    self.selected = (self.selected + self.matches.len() - 1) % self.matches.len();
                }
            }
            KeyCode::Down => {
                if !self.matches.is_empty() {
                    self.selected = (self.selected + 1) % self.matches.len();
                }
            }
            KeyCode::Enter => {
                return self
                    .matches
                    .get(self.selected)
                    .copied()
                    .map(Transition::Open);
            }
            // ── app ──
            KeyCode::Esc => return Some(Transition::Quit),
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

            // Row 0: prompt input (editable) — caret shown as a block
            if inner.height >= 1 {
                let prompt_area = Rect {
                    x: inner.x,
                    y: inner.y,
                    width: inner.width,
                    height: 1,
                };

                // If query is empty show the placeholder, else show the typed text
                let mut spans: Vec<Span> = vec![];
                if self.query.is_empty() {
                    // Block cursor (opencode: cursor color = theme.text)
                    spans.push(Span::styled(
                        " ",
                        Style::new()
                            .bg(theme::text())
                            .fg(theme::page_bg())
                            .add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::styled(
                        "Ask anything… or choose an operation",
                        Style::new().fg(theme::text_muted()),
                    ));
                } else {
                    // Split at caret for a block cursor visualization
                    let caret = self.caret.min(self.query.len());
                    let (before, after) = self.query.split_at(caret);
                    if !before.is_empty() {
                        spans.push(Span::styled(
                            before.to_string(),
                            Style::new().fg(theme::text()),
                        ));
                    }
                    if let Some(ch) = after.chars().next() {
                        spans.push(Span::styled(
                            ch.to_string(),
                            Style::new()
                                .bg(theme::text())
                                .fg(theme::page_bg())
                                .add_modifier(Modifier::BOLD),
                        ));
                        spans.push(Span::styled(
                            after[ch.len_utf8()..].to_string(),
                            Style::new().fg(theme::text()),
                        ));
                    } else {
                        // Caret at end — solid text-coloured block
                        spans.push(Span::styled(
                            " ",
                            Style::new()
                                .bg(theme::text())
                                .fg(theme::page_bg())
                                .add_modifier(Modifier::BOLD),
                        ));
                    }
                }
                frame.render_widget(Paragraph::new(Line::from(spans)), prompt_area);
            }

            // Row 1: horizontal `·` menu of *filtered* operations
            if inner.height >= 2 {
                let mut spans: Vec<Span> = vec![];
                if self.matches.is_empty() {
                    spans.push(Span::styled(
                        "no matches",
                        Style::new()
                            .fg(theme::text_muted())
                            .add_modifier(Modifier::ITALIC),
                    ));
                } else {
                    for (idx, op) in self.matches.iter().enumerate() {
                        if idx > 0 {
                            spans.push(Span::styled(" · ", Style::new().fg(theme::text_muted())));
                        }
                        let title = short_title(op);
                        let style = if idx == self.selected {
                            Style::new().fg(theme::text()).add_modifier(Modifier::BOLD)
                        } else {
                            Style::new().fg(theme::text_muted())
                        };
                        spans.push(Span::styled(title, style));
                    }
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
            let hint_text = "↑↓ select  enter run  Esc quit";
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
                Span::styled(
                    "↑↓",
                    Style::new().fg(theme::text()).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" select  ", Style::new().fg(theme::text_muted())),
                Span::styled(
                    "enter",
                    Style::new().fg(theme::text()).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" run  ", Style::new().fg(theme::text_muted())),
                Span::styled(
                    "Esc",
                    Style::new().fg(theme::text()).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" quit", Style::new().fg(theme::text_muted())),
            ]);
            frame.render_widget(Paragraph::new(hint_line).right_aligned(), hint_area);
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
                Span::styled(
                    "● Tip ",
                    Style::new()
                        .fg(theme::warning())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Drag and drop a PDF file or pass a path as an argument",
                    Style::new().fg(theme::text_muted()),
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
                        Style::new().fg(theme::text()).add_modifier(Modifier::BOLD),
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
                        Style::new().fg(theme::text_muted()),
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

// ---------------------------------------------------------------------------
// Theme picker — opencode-style list of themes, switchable at runtime.
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct ThemePickerScreen {
    names: Vec<String>,
    selected: usize,
}

impl ThemePickerScreen {
    pub fn new() -> Self {
        let names = theme::names();
        Self { names, selected: 0 }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> Option<Transition> {
        use ratatui::crossterm::event::KeyModifiers;
        match key.code {
            KeyCode::Up => {
                if !self.names.is_empty() {
                    self.selected = (self.selected + self.names.len() - 1) % self.names.len();
                }
            }
            KeyCode::Down => {
                if !self.names.is_empty() {
                    self.selected = (self.selected + 1) % self.names.len();
                }
            }
            KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(name) = self.names.get(self.selected) {
                    theme::set_active(name);
                }
                return Some(Transition::Back);
            }
            KeyCode::Enter => {
                if let Some(name) = self.names.get(self.selected) {
                    theme::set_active(name);
                }
                return Some(Transition::Back);
            }
            KeyCode::Esc | KeyCode::Char('q') => return Some(Transition::Back),
            _ => {}
        }
        None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let list_w: u16 = 46.min(area.width.saturating_sub(4));
        let rows = self.names.len() as u16;
        let box_h = (rows + 3).min(area.height.saturating_sub(2)); // title + list + pad

        let box_x = area.x + (area.width.saturating_sub(list_w)) / 2;
        let stack_h = box_h.saturating_add(1).saturating_add(1);
        let start_y = if area.height > stack_h {
            area.y + (area.height - stack_h) / 2
        } else {
            area.y
        };

        let box_area = Rect {
            x: box_x,
            y: start_y,
            width: list_w,
            height: box_h,
        };
        let inner = render_prompt_box(frame, box_area);

        // Title row
        if inner.height >= 1 {
            frame.render_widget(
                Paragraph::new(Line::styled(
                    "theme",
                    Style::new().fg(theme::text()).add_modifier(Modifier::BOLD),
                )),
                Rect {
                    x: inner.x,
                    y: inner.y,
                    width: inner.width,
                    height: 1,
                },
            );
        }

        // Theme rows with a small colour swatch
        let list_top = inner.y + 1;
        let visible = inner.height.saturating_sub(1) as usize;
        for (i, name) in self.names.iter().enumerate() {
            if i >= visible {
                break;
            }
            let selected = i == self.selected;
            let mut spans: Vec<Span> = vec![];
            spans.push(Span::styled(
                if selected { "›" } else { " " },
                if selected {
                    theme::accent().add_modifier(Modifier::BOLD)
                } else {
                    Style::new()
                },
            ));
            spans.push(Span::raw(" "));
            if let Some(t) = theme::get(name) {
                spans.push(Span::styled("███", Style::new().fg(t.accent)));
                spans.push(Span::styled(" ", Style::new()));
                spans.push(Span::styled("███", Style::new().fg(t.primary)));
                spans.push(Span::styled(" ", Style::new()));
            }
            spans.push(Span::styled(
                name.clone(),
                if selected {
                    theme::accent_bold()
                } else {
                    Style::new().fg(theme::text_muted())
                },
            ));
            frame.render_widget(
                Paragraph::new(Line::from(spans)),
                Rect {
                    x: inner.x,
                    y: list_top + i as u16,
                    width: inner.width,
                    height: 1,
                },
            );
        }
    }
}
