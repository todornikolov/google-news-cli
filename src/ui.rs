//! The results screen: a scrollable list, the selected link at the bottom, Enter opens it.

use ratatui::crossterm::event::{self, KeyCode};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::{DefaultTerminal, Frame};

use crate::Hit;

pub struct App {
    heading: String,
    hits: Vec<Hit>,
    list: ListState,
}

/// What a key press asks the event loop to do.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    Continue,
    Quit,
    Open(String),
}

impl App {
    pub fn new(heading: String, hits: Vec<Hit>) -> Self {
        Self {
            heading,
            hits,
            list: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            let Some(key) = event::read()?.as_key_press_event() else {
                continue; // mouse, resize, key release...
            };
            match self.on_key(key.code) {
                Outcome::Quit => return Ok(()),
                Outcome::Open(link) => open::that_detached(link)?,
                Outcome::Continue => {}
            }
        }
    }

    /// Update the selection for a key press and say what the loop should do next.
    /// Pure of I/O, so it can be tested without a terminal or a browser.
    fn on_key(&mut self, code: KeyCode) -> Outcome {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return Outcome::Quit,
            KeyCode::Down | KeyCode::Char('j') => self.list.select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.list.select_previous(),
            KeyCode::Enter => {
                if let Some(hit) = self.selected() {
                    return Outcome::Open(hit.link.clone());
                }
            }
            _ => {}
        }
        Outcome::Continue
    }

    fn selected(&self) -> Option<&Hit> {
        self.list.selected().and_then(|i| self.hits.get(i))
    }

    fn render(&mut self, frame: &mut Frame) {
        let [header, body, link, keys] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        frame.render_widget(Line::from(self.heading.as_str()).bold(), header);

        let items = self.hits.iter().enumerate().map(|(i, hit)| {
            ListItem::new(vec![
                Line::from(format!("{:>2}. {}", i + 1, hit.title)).bold(),
                Line::from(format!("    {}", hit.detail)).dim(),
                Line::default(),
            ])
        });
        let list = List::new(items)
            .highlight_style(Style::new().yellow())
            .highlight_symbol("▌");
        frame.render_stateful_widget(list, body, &mut self.list);

        let url = self.selected().map_or("", |hit| hit.link.as_str());
        frame.render_widget(Line::from(url).cyan().underlined(), link);
        frame.render_widget(Line::from("↑↓/jk move · Enter open · q quit").dim(), keys);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    fn sample_hits() -> Vec<Hit> {
        vec![
            Hit {
                title: "Ratatui".into(),
                link: "https://ratatui.rs".into(),
                detail: "Terminal UIs in Rust".into(),
            },
            Hit {
                title: "Crossterm".into(),
                link: "https://docs.rs/crossterm".into(),
                detail: "Terminal library".into(),
            },
        ]
    }

    fn selected_title(app: &App) -> &str {
        &app.selected().unwrap().title
    }

    #[test]
    fn arrows_and_vim_keys_move_the_selection() {
        for down in [KeyCode::Down, KeyCode::Char('j')] {
            let mut app = App::new("Google · rust tui".into(), sample_hits());
            assert_eq!(app.on_key(down), Outcome::Continue);
            assert_eq!(selected_title(&app), "Crossterm");
        }
        for up in [KeyCode::Up, KeyCode::Char('k')] {
            let mut app = App::new("Google · rust tui".into(), sample_hits());
            app.on_key(KeyCode::Down);
            assert_eq!(app.on_key(up), Outcome::Continue);
            assert_eq!(selected_title(&app), "Ratatui");
        }
    }

    #[test]
    fn q_and_escape_quit() {
        let mut app = App::new("Google · rust tui".into(), sample_hits());

        assert_eq!(app.on_key(KeyCode::Char('q')), Outcome::Quit);
        assert_eq!(app.on_key(KeyCode::Esc), Outcome::Quit);
    }

    #[test]
    fn enter_opens_the_selected_link() {
        let mut app = App::new("Google · rust tui".into(), sample_hits());
        app.on_key(KeyCode::Down);

        assert_eq!(
            app.on_key(KeyCode::Enter),
            Outcome::Open("https://docs.rs/crossterm".into())
        );
    }

    #[test]
    fn enter_with_no_results_does_nothing() {
        let mut app = App::new("Google · rust tui".into(), vec![]);

        assert_eq!(app.on_key(KeyCode::Enter), Outcome::Continue);
    }

    #[test]
    fn other_keys_are_ignored() {
        let mut app = App::new("Google · rust tui".into(), sample_hits());

        assert_eq!(app.on_key(KeyCode::Char('x')), Outcome::Continue);
        assert_eq!(selected_title(&app), "Ratatui");
    }

    #[test]
    fn shows_results_and_the_selected_link() {
        let mut app = App::new("Google · rust tui".into(), sample_hits());
        let mut terminal = Terminal::new(TestBackend::new(50, 12)).unwrap();

        app.list.select_next();
        terminal.draw(|frame| app.render(frame)).unwrap();

        let screen: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        for expected in [
            "Google · rust tui",
            " 1. Ratatui",
            " 2. Crossterm",
            "https://docs.rs/crossterm",
        ] {
            assert!(
                screen.contains(expected),
                "missing {expected:?} in:\n{screen}"
            );
        }
    }
}
