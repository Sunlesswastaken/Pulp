use std::fmt::Display;

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::Frame;

use crate::tui::theme;

/// Keyboard-navigable list for the centered card.
///
/// Visual contract (OpenCode / slate-cyan):
/// - unselected rows: dim slate gray (#64748B)
/// - selected row:   crisp white (#F8FAFC) bold
/// - highlight marker `▸` in cyan (#38BDF8)
///
/// The surrounding card (`Block::bordered().border_type(Rounded)`) is owned
/// by the screen; this widget only owns the rows.
#[derive(Debug)]
pub struct NavList<T> {
    items: Vec<T>,
    state: ListState,
}

impl<T> NavList<T>
where
    T: Display + Clone,
{
    pub fn new(_title: &'static str, items: impl IntoIterator<Item = T>) -> Self {
        let items: Vec<T> = items.into_iter().collect();
        let mut state = ListState::default();
        state.select((!items.is_empty()).then_some(0));
        Self { items, state }
    }

    pub fn selected(&self) -> Option<&T> {
        self.state.selected().and_then(|index| self.items.get(index))
    }

    /// Wrap-around next (OpenCode-style: bottom wraps to top).
    pub fn select_next(&mut self) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let next = match self.state.selected() {
            Some(i) if i + 1 < len => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.state.select(Some(next));
    }

    pub fn select_previous(&mut self) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let prev = match self.state.selected() {
            Some(0) | None => len - 1,
            Some(i) => i - 1,
        };
        self.state.select(Some(prev));
    }

    /// Render inside `area` (which should already be the inner area of the
    /// rounded-bordered card). Handles styling of selected vs muted rows.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.render_in(frame, area);
    }

    /// Backwards-compatible helper: old call sites passed (title_area, list_area).
    /// We ignore the title_area and render into list_area.
    pub fn render_split(&self, frame: &mut Frame, _title_area: Rect, list_area: Rect) {
        self.render(frame, list_area);
    }

    fn render_in(&self, frame: &mut Frame, area: Rect) {
        let selected = self.state.selected();

        let rows: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let is_selected = Some(idx) == selected;
                if is_selected {
                    // Selected: white bold. Marker is supplied via highlight_symbol.
                    ListItem::new(Line::from(Span::styled(
                        item.to_string(),
                        theme::selected(),
                    )))
                } else {
                    ListItem::new(Line::from(Span::styled(
                        item.to_string(),
                        theme::muted(),
                    )))
                }
            })
            .collect();

        let list = List::new(rows)
            .highlight_symbol(Span::styled("▸ ", theme::highlight_marker()))
            .highlight_style(theme::selected())
            // Ensure the list itself doesn't add extra style that would overwrite row styles
            .style(Style::default());

        let mut state = self.state.clone();
        frame.render_stateful_widget(list, area, &mut state);
    }
}
