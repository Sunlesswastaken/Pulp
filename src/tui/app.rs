use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::DefaultTerminal;

use crate::error::Result;

use super::components::footer as app_footer;
use super::screens::{HomeScreen, OperationScreen, Screen, Transition};

/// Poll interval for input events; short enough to stay responsive,
/// long enough to keep the CPU quiet.
const TICK: Duration = Duration::from_millis(100);

pub fn run() -> Result<()> {
    let mut terminal = ratatui::init();
    let result = App::run_to_completion(&mut terminal);
    ratatui::restore();
    result
}

/// The TUI shell: owns the current screen and drives the event loop.
struct App {
    home: HomeScreen,
    screen: Screen,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            home: HomeScreen::new(),
            screen: Screen::Home,
            should_quit: false,
        }
    }

    fn run_to_completion(terminal: &mut DefaultTerminal) -> Result<()> {
        let mut app = Self::new();
        while !app.should_quit {
            terminal.draw(|frame| app.render(frame))?;
            app.handle_events()?;
        }
        Ok(())
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();

        // Reserve the absolute bottom line for the pinned global footer
        // (cwd:branch on the left, version on the right). Everything else
        // is vertically + horizontally centered within the remaining area.
        let (content_area, footer_area) = split_content_and_footer(area);

        match &self.screen {
            Screen::Home => self.home.render(frame, content_area),
            Screen::Operation(screen) => screen.render(frame, content_area),
        }

        // Global footer is always visible, regardless of screen.
        app_footer::render(frame, footer_area);
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(TICK)? {
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        let transition = match &mut self.screen {
            Screen::Home => self.home.on_key(key),
            Screen::Operation(screen) => screen.on_key(key),
        };

        match transition {
            Some(Transition::Quit) => self.should_quit = true,
            Some(Transition::Open(operation)) => {
                self.screen = Screen::Operation(OperationScreen::new(operation))
            }
            Some(Transition::Back) => self.screen = Screen::Home,
            None => {}
        }
    }
}

fn split_content_and_footer(area: Rect) -> (Rect, Rect) {
    if area.height == 0 {
        return (area, area);
    }
    if area.height == 1 {
        // No room for content — footer takes the only line.
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
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    let content = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height.saturating_sub(1),
    };
    (content, footer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::operation::Operation;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn rendered(app: &App) -> String {
        // Use a tall enough backend so the vertically-centered stack (header 6
        // + card 11 + hints 1 + tip 1 = ~23 rows) is fully visible plus footer.
        let mut terminal = Terminal::new(TestBackend::new(80, 30)).unwrap();
        terminal.draw(|frame| app.render(frame)).unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    #[test]
    fn home_screen_shows_operations() {
        let app = App::new();
        let text = rendered(&app);
        // Header is opencode-style block (contains █) and the centered card
        // now shows the horizontal `·` menu with prompt.
        assert!(
            text.contains('█') || text.contains("PULP") || text.contains("P U L P"),
            "home screen missing block ASCII header (expected █ or PULP)"
        );
        assert!(
            text.contains("Ask anything") || text.contains("Select action"),
            "home screen missing prompt text"
        );
        assert!(text.contains('·'), "home screen missing `·` separator in horizontal menu");
        for expected in [
            "Compress",
            "Merge",
            "Split",
            "Remove",
            "Extract",
            "Password",
            "Info",
        ] {
            assert!(text.contains(expected), "home screen missing `{expected}`");
        }
    }

    #[test]
    fn home_screen_shows_key_hints() {
        let app = App::new();
        let text = rendered(&app);
        // New menu shows `↑↓ select  enter run` right-aligned under the card
        for expected in ["select", "run"] {
            assert!(text.contains(expected), "home screen missing hint `{expected}`");
        }
    }

    #[test]
    fn home_screen_shows_tip() {
        let app = App::new();
        let text = rendered(&app);
        assert!(
            text.contains("Drag and drop") || text.contains("Tip:"),
            "home screen missing tip line"
        );
    }

    #[test]
    fn home_screen_shows_footer() {
        let app = App::new();
        let text = rendered(&app);
        // Right side version tag, left side may contain cwd/branch.
        assert!(
            text.contains("v0.1.0"),
            "footer missing version tag (expected v0.1.0)"
        );
    }

    #[test]
    fn operation_screen_shows_breadcrumb_and_status() {
        let mut app = App::new();
        app.screen = Screen::Operation(OperationScreen::new(Operation::Compress));
        let text = rendered(&app);
        assert!(text.contains("pulp"), "missing brand");
        assert!(text.contains("compress"), "missing breadcrumb section");
        assert!(text.contains("not implemented yet"), "missing status");
    }

    #[test]
    fn selecting_operation_opens_its_screen() {
        let mut app = App::new();
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(matches!(app.screen, Screen::Operation(_)));
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(matches!(app.screen, Screen::Home));
    }

    #[test]
    fn q_quits_from_home() {
        let mut app = App::new();
        app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        assert!(app.should_quit);
    }



























    #[test]
    fn keyboard_navigation_wraps() {
        let mut app = App::new();
        // From top, Up should wrap to bottom (Info).
        app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        // Pressing Enter should then open Info.
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        match &app.screen {
            Screen::Operation(s) => assert_eq!(s.operation, Operation::Info),
            _ => panic!("expected Info operation screen after wrap-around Up + Enter"),
        }
    }
}
