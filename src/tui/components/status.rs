use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Success,
    Error,
}

/// A single status line with an unambiguous symbol and tone:
/// `• info`, `✓ success`, `✗ error`.
#[derive(Debug, Clone)]
pub struct Status {
    kind: StatusKind,
    message: String,
}

impl Status {
    pub fn info(message: impl Into<String>) -> Self {
        Self { kind: StatusKind::Info, message: message.into() }
    }

    #[allow(dead_code)] // used by operation completion/error screens (next phase)
    pub fn success(message: impl Into<String>) -> Self {
        Self { kind: StatusKind::Success, message: message.into() }
    }

    #[allow(dead_code)] // used by operation error screens (next phase)
    pub fn error(message: impl Into<String>) -> Self {
        Self { kind: StatusKind::Error, message: message.into() }
    }

    pub fn line(&self) -> Line<'static> {
        let (symbol, style) = match self.kind {
            StatusKind::Info => ('•', theme::accent()),
            StatusKind::Success => ('✓', theme::accent_bold()),
            StatusKind::Error => ('✗', theme::error()),
        };
        let mut spans = vec![Span::styled(format!("{symbol} "), style)];
        spans.push(Span::styled(self.message.clone(), style));
        Line::from(spans)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Paragraph::new(self.line()), area);
    }
}