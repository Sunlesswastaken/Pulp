use std::env;
use std::path::{Path, PathBuf};

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::theme;

/// Pinned single-line footer at the absolute bottom of the terminal.
///
/// Left:  current working directory + git branch, e.g. `~/projects/pulp:main`
/// Right: version tag, e.g. `v0.1.0`
///
/// Rendered in dim slate gray (#64748B) so it stays out of the way.
pub fn render(frame: &mut Frame, area: Rect) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let left = cwd_with_branch();
    let right = format!("v{}", env!("CARGO_PKG_VERSION"));

    // How much space we actually have. Truncate left if needed, never truncate right.
    let right_len = right.len() as u16;
    let available = area.width.saturating_sub(right_len).saturating_sub(1); // 1 for gap
    let left_display = if (left.len() as u16) > available && available > 3 {
        let mut truncated = left.clone();
        truncated.truncate((available.saturating_sub(3)) as usize);
        format!("{truncated}...")
    } else {
        left
    };

    let line = Line::from(vec![
        Span::styled(left_display, theme::muted()),
        Span::raw(" "),
    ]);

    // We render a single line with left text and right-aligned version via two paragraphs.
    // Simpler: one paragraph with left, and a second right-aligned paragraph overlay.
    let left_para = Paragraph::new(line).left_aligned();
    frame.render_widget(left_para, area);

    // Right-aligned version tag — render in a small rect on the right edge.
    let right_width = right_len.min(area.width);
    let right_area = Rect {
        x: area.x + area.width.saturating_sub(right_width),
        y: area.y,
        width: right_width,
        height: 1,
    };
    let right_para = Paragraph::new(Line::styled(right, theme::muted())).right_aligned();
    frame.render_widget(right_para, right_area);
}

// ---------------------------------------------------------------------------
// Helpers: cwd + git branch
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
    let cwd_str = cwd.display().to_string();

    // Collapse $HOME to ~ for brevity — matches the spec's `~/projects/pulp`.
    if let Ok(home) = env::var("HOME") {
        if cwd_str == home {
            return "~".to_string();
        }
        if let Some(stripped) = cwd_str.strip_prefix(&format!("{home}/")) {
            return format!("~/{stripped}");
        }
    }

    // Also handle HOME not set or Windows.
    cwd_str
}

fn git_branch_for_cwd() -> Option<String> {
    let cwd = env::current_dir().ok()?;
    let git_head = find_git_head(&cwd)?;
    let content = std::fs::read_to_string(&git_head).ok()?;
    let content = content.trim();

    // Typical: "ref: refs/heads/main"
    if let Some(ref_path) = content.strip_prefix("ref: ") {
        // Branch is the last component after slash.
        let branch = ref_path.rsplit('/').next().unwrap_or(ref_path);
        if !branch.is_empty() {
            return Some(branch.to_string());
        }
        return None;
    }

    // Detached HEAD: content is a commit sha — show short sha.
    if content.len() >= 7 && content.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(content[..7].to_string());
    }

    None
}

fn find_git_head(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);

    while let Some(dir) = current {
        let git_path = dir.join(".git");

        if git_path.is_file() {
            // Worktree / submodule: .git is a file containing "gitdir: <path>"
            if let Ok(text) = std::fs::read_to_string(&git_path) {
                if let Some(gitdir) = text.strip_prefix("gitdir:") {
                    let gitdir = PathBuf::from(gitdir.trim());
                    // Resolve relative to `dir`
                    let resolved = if gitdir.is_absolute() {
                        gitdir
                    } else {
                        dir.join(gitdir)
                    };
                    let head = resolved.join("HEAD");
                    if head.is_file() {
                        return Some(head);
                    }
                }
            }
        } else if git_path.is_dir() {
            let head = git_path.join("HEAD");
            if head.is_file() {
                return Some(head);
            }
        }

        current = dir.parent();
    }

    None
}

// ---------------------------------------------------------------------------
// Hint bar — muted shortcut labels beneath the card (not the bottom footer).
// This used to be `Footer`; kept as `HintBar` for backward compatibility.
// ---------------------------------------------------------------------------

/// Muted shortcut hint bar rendered beneath the centered card, e.g.
/// `↑↓ navigate   enter select   q quit`.
///
/// Centered, slate gray, keys in bold, descriptions in italic.
#[derive(Debug, Clone, Default)]
pub struct HintBar {
    hints: Vec<(String, String)>,
}

impl HintBar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hint(mut self, key: impl Into<String>, description: impl Into<String>) -> Self {
        self.hints.push((key.into(), description.into()));
        self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if area.height == 0 {
            return;
        }
        let mut spans: Vec<Span> = Vec::new();
        for (index, (key, description)) in self.hints.iter().enumerate() {
            if index > 0 {
                spans.push(Span::styled("   ", theme::muted()));
            }
            spans.push(Span::styled(key.clone(), theme::muted_bold()));
            spans.push(Span::styled(format!(" {description}"), theme::muted()));
        }
        let para = Paragraph::new(Line::from(spans)).centered();
        frame.render_widget(para, area);
    }
}

/// Legacy alias — `Footer` was previously the hint bar. Keep it compiling.
pub type Footer = HintBar;

/// Legacy helper: indented() was used to inset hint bar — now centred.
#[allow(dead_code)]
pub fn indented(area: Rect) -> Rect {
    area
}
