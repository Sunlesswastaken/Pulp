//! OpenCode-inspired single-file TUI for `pulp`
//!
//! Run with:
//!   cargo run --example opencode_ui
//!
//! This file is intentionally self-contained — copy it to `src/main.rs`
//! if you want a one-file `pulp` TUI. It demonstrates every aesthetic
//! requirement from the spec:
//!   - ratatui 0.30 + crossterm 0.28
//!   - vertically & horizontally centered layout
//!   - pinned bottom footer (cwd:branch left, version right)
//!   - slate / cyan theme, block ASCII header, rounded card,
//!     muted hints, centered tip, graceful teardown.

use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, ListState, Paragraph};
use ratatui::{DefaultTerminal, Frame};

// ---------------------------------------------------------------------------
// Theme — exact hex values from the spec
// ---------------------------------------------------------------------------
const SLATE: Color = Color::Rgb(100, 116, 139); // #64748B
const SLATE_300: Color = Color::Rgb(203, 213, 225); // #CBD5E1 — requested header
const WHITE: Color = Color::Rgb(248, 250, 252); // #F8FAFC
const CYAN: Color = Color::Rgb(56, 189, 248); // #38BDF8

fn muted() -> Style {
    Style::default().fg(SLATE)
}
fn muted_bold() -> Style {
    Style::default().fg(SLATE).add_modifier(Modifier::BOLD)
}
fn accent_bold() -> Style {
    Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
}
fn title_style() -> Style {
    Style::default().fg(WHITE).add_modifier(Modifier::BOLD)
}
fn header_style() -> Style {
    Style::default()
        .fg(SLATE_300)
        .add_modifier(Modifier::BOLD)
}
fn selected_style() -> Style {
    Style::default().fg(WHITE).add_modifier(Modifier::BOLD)
}
fn border_style() -> Style {
    Style::default().fg(SLATE)
}

// ---------------------------------------------------------------------------
// Data
// ---------------------------------------------------------------------------
const OPERATIONS: [&str; 7] = [
    "Compress",
    "Merge",
    "Split",
    "Remove pages",
    "Extract pages",
    "Password",
    "Info",
];

// Opencode-style PULP — 4×19 block, left muted / right Slate 300 + gap 1
// Mirrors https://github.com/anomalyco/opencode `packages/tui/src/logo.ts`
// Bottom row uses "█▀▀▀" for P so stem isn't cutoff
const PULP_LEFT: [&str; 4] = ["         ", "█▀▀█ █  █", "█  █ █  █", "█▀▀▀ ▀▀▀▀"];
const PULP_RIGHT: [&str; 4] = ["         ", "█    █▀▀█", "█    █  █", "▀▀▀▀ █▀▀▀"];

const VERSION: &str = env!("CARGO_PKG_VERSION");

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------
struct App {
    state: ListState,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            should_quit: false,
        }
    }

    fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    fn next(&mut self) {
        let next = match self.selected() {
            Some(i) => (i + 1) % OPERATIONS.len(),
            None => 0,
        };
        self.state.select(Some(next));
    }

    fn previous(&mut self) {
        let prev = match self.selected() {
            Some(0) | None => OPERATIONS.len() - 1,
            Some(i) => i - 1,
        };
        self.state.select(Some(prev));
    }
}

// ---------------------------------------------------------------------------
// Entry
// ---------------------------------------------------------------------------
fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore(); // leaves alternate screen, disables raw mode, shows cursor
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();
    const TICK: Duration = Duration::from_millis(100);

    while !app.should_quit {
        terminal.draw(|frame| render(&app, frame))?;

        if event::poll(TICK)? {
            if let Event::Key(key) = event::read()? {
                // Only handle Press/Repeat to avoid double firing on release
                if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                    continue;
                }
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.code == KeyCode::Char('c')
                {
                    app.should_quit = true;
                    continue;
                }
                match key.code {
                    KeyCode::Up | KeyCode::Left | KeyCode::Char('k') | KeyCode::Char('h') => {
                        app.previous()
                    }
                    KeyCode::Down | KeyCode::Right | KeyCode::Char('j') | KeyCode::Char('l') => {
                        app.next()
                    }
                    KeyCode::Enter => {
                        if let Some(idx) = app.selected() {
                            // In a real pulp build this would dispatch to
                            // `operations::run(OPERATIONS[idx])`.
                            // For the demo we just quit to show selection works.
                            // Replace with your dispatch:
                            //   handle_operation(OPERATIONS[idx]);
                            let _ = idx; // suppress unused warning
                            // app.should_quit = true;
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => app.should_quit = true,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Rendering — vertically & horizontally centered, footer pinned
// ---------------------------------------------------------------------------
fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    if area.width == 0 || area.height == 0 {
        return;
    }

    // Reserve the absolute bottom line for the global footer
    let (content_area, footer_area) = split_content_and_footer(area);

    // --- Centered stack layout (header + horizontal menu card + hints + tip) ---
    let header_h: u16 = if content_area.width < 19 || content_area.height < 4 {
        1
    } else {
        4
    };
    // Horizontal menu card — 2 rows + borders = 4
    let ops_line: u16 = {
        // short titles like the requested "Compress · Merge · …"
        let shorts = ["Compress", "Merge", "Split", "Remove", "Extract", "Password", "Info"];
        let mut len = 2; // indent "  "
        for (i, s) in shorts.iter().enumerate() {
            if i > 0 {
                len += 3; // " · "
            }
            len += s.len();
        }
        len as u16
    };
    let prompt_len: u16 = "▌ Select action.. (or search)".len() as u16;
    let inner_needed = ops_line.max(prompt_len).saturating_add(4);
    let card_width: u16 = (inner_needed + 2)
        .min(content_area.width.saturating_sub(4))
        .max(40);
    let card_height: u16 = 4;
    let hint_h: u16 = 1;
    let tip_h: u16 = 1;

    let stack_h = header_h + 1 + card_height + 1 + hint_h + 2 + tip_h;
    let start_y = if content_area.height > stack_h {
        content_area.y + (content_area.height - stack_h) / 2
    } else {
        content_area.y
    };
    let card_x = content_area.x + (content_area.width.saturating_sub(card_width)) / 2;

    let mut y = start_y;

    // Header — opencode-style left muted / right Slate 300 + gap 1
    let header_area = Rect {
        x: content_area.x,
        y,
        width: content_area.width,
        height: header_h,
    };
    if header_h == 1 {
        frame.render_widget(
            Paragraph::new(Line::styled("PULP", header_style())).centered(),
            header_area,
        );
    } else {
        let lines: Vec<Line> = (0..4)
            .map(|i| {
                Line::from(vec![
                    Span::styled(PULP_LEFT[i], muted()),
                    Span::raw(" "),
                    Span::styled(PULP_RIGHT[i], header_style()),
                ])
            })
            .collect();
        frame.render_widget(Paragraph::new(lines).centered(), header_area);
    }
    y += header_h + 1;

    // Card — centered, rounded borders, 2 rows: prompt + horizontal · menu
    if y + card_height <= content_area.y + content_area.height {
        let card_area = Rect {
            x: card_x,
            y,
            width: card_width,
            height: card_height,
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style());
        frame.render_widget(block, card_area);

        let inner = card_area.inner(Margin {
            vertical: 1,
            horizontal: 2,
        });
        if inner.width > 0 && inner.height >= 2 {
            let prompt_area = Rect {
                x: inner.x,
                y: inner.y,
                width: inner.width,
                height: 1,
            };
            let prompt_line = Line::from(vec![
                Span::styled("▌ ", accent_bold()),
                Span::styled("Select action.. (or search)", muted()),
            ]);
            frame.render_widget(Paragraph::new(prompt_line), prompt_area);

            let ops_area = Rect {
                x: inner.x,
                y: inner.y + 1,
                width: inner.width,
                height: 1,
            };
            let shorts = ["Compress", "Merge", "Split", "Remove", "Extract", "Password", "Info"];
            let mut spans: Vec<Span> = vec![Span::raw("  ")];
            for (idx, title) in shorts.iter().enumerate() {
                if idx > 0 {
                    spans.push(Span::styled(" · ", muted()));
                }
                let is_selected = Some(idx) == app.selected();
                let style = if is_selected {
                    selected_style()
                } else {
                    muted()
                };
                spans.push(Span::styled(*title, style));
            }
            frame.render_widget(Paragraph::new(Line::from(spans)), ops_area);
        }
        y += card_height + 1;
    }

    // Hints — right-aligned under the card, as in the requested ascii
    if y + hint_h <= content_area.y + content_area.height {
        let hint_text = "↑↓ select  enter run";
        let hint_width = hint_text.len() as u16;
        let hint_x = card_x.saturating_add(card_width).saturating_sub(hint_width);
        let hint_area = Rect {
            x: hint_x,
            y,
            width: hint_width.min(area.width.saturating_sub(hint_x - area.x)),
            height: hint_h,
        };
        let hint_line = Line::from(vec![
            Span::styled("↑↓", muted_bold()),
            Span::styled(" select  ", muted()),
            Span::styled("enter", muted_bold()),
            Span::styled(" run", muted()),
        ]);
        frame.render_widget(Paragraph::new(hint_line).right_aligned(), hint_area);
        y += hint_h + 2;
    }

    // Tip — centered, cyan bullet
    if y + tip_h <= content_area.y + content_area.height {
        let tip_area = Rect {
            x: content_area.x,
            y,
            width: content_area.width,
            height: tip_h,
        };
        let tip = Line::from(vec![
            Span::styled("• ", accent_bold()),
            Span::styled("Tip: ", muted_bold()),
            Span::styled(
                "Drag and drop a PDF file or pass a path as an argument",
                muted(),
            ),
        ]);
        frame.render_widget(Paragraph::new(tip).centered(), tip_area);
    }

    // Pinned footer — always at absolute bottom
    render_footer(frame, footer_area);
}

fn split_content_and_footer(area: Rect) -> (Rect, Rect) {
    if area.height <= 1 {
        return (
            Rect {
                height: 0,
                ..area
            },
            area,
        );
    }
    let footer = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };
    let content = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height - 1,
    };
    (content, footer)
}

fn render_footer(frame: &mut Frame, area: Rect) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let left = cwd_with_branch();
    let right = format!("v{VERSION}");
    let right_len = right.len() as u16;
    let available = area.width.saturating_sub(right_len + 1);
    let left_display = if (left.len() as u16) > available && available > 3 {
        let mut s = left.clone();
        s.truncate((available - 3) as usize);
        format!("{s}...")
    } else {
        left
    };

    let left_para = Paragraph::new(Line::styled(left_display, muted())).left_aligned();
    frame.render_widget(left_para, area);

    let right_width = right_len.min(area.width);
    let right_area = Rect {
        x: area.x + area.width.saturating_sub(right_width),
        y: area.y,
        width: right_width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Line::styled(right, muted())).right_aligned(),
        right_area,
    );
}

// ---------------------------------------------------------------------------
// Helpers — cwd + git branch (matches spec: `~/projects/pulp:main`)
// ---------------------------------------------------------------------------
fn cwd_with_branch() -> String {
    let cwd = current_dir_display();
    match git_branch_for_cwd() {
        Some(branch) => format!("{cwd}:{branch}"),
        None => cwd,
    }
}

fn current_dir_display() -> String {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let display = cwd.display().to_string();
    if let Ok(home) = env::var("HOME") {
        if display == home {
            return "~".to_string();
        }
        if let Some(stripped) = display.strip_prefix(&format!("{home}/")) {
            return format!("~/{stripped}");
        }
    }
    display
}

fn git_branch_for_cwd() -> Option<String> {
    let cwd = env::current_dir().ok()?;
    let head = find_git_head(&cwd)?;
    let content = std::fs::read_to_string(head).ok()?;
    let content = content.trim();
    if let Some(ref_path) = content.strip_prefix("ref: ") {
        return Some(
            ref_path
                .rsplit('/')
                .next()
                .unwrap_or(ref_path)
                .to_string(),
        );
    }
    if content.len() >= 7 && content.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(content[..7].to_string());
    }
    None
}

fn find_git_head(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let git = dir.join(".git");
        if git.is_file() {
            if let Ok(text) = std::fs::read_to_string(&git) {
                if let Some(gitdir) = text.strip_prefix("gitdir:") {
                    let p = PathBuf::from(gitdir.trim());
                    let resolved = if p.is_absolute() { p } else { dir.join(p) };
                    let head = resolved.join("HEAD");
                    if head.is_file() {
                        return Some(head);
                    }
                }
            }
        } else if git.is_dir() {
            let head = git.join("HEAD");
            if head.is_file() {
                return Some(head);
            }
        }
        current = dir.parent();
    }
    None
}
