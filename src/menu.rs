use crate::games::{Game, get_all_games};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use std::io;

pub struct Menu {
    selected_index: u32,
    selectable_games: Vec<Box<dyn Game>>,
    quit: bool,
    in_game: bool,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            selectable_games: get_all_games(),
            quit: false,
            in_game: false,
        }
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        while !self.quit {
            while !self.in_game {
                terminal.draw(|frame| {
                    let size = frame.area();

                    let block = Block::default()
                        .title("Game Selector")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::White));

                    let items: Vec<ListItem> = self
                        .selectable_games
                        .iter()
                        .map(|game| {
                            ListItem::new(vec![
                                Line::from(Span::styled(
                                    game.name(),
                                    Style::default()
                                        .fg(Color::Yellow)
                                        .add_modifier(Modifier::BOLD),
                                )),
                                Line::from(Span::styled(
                                    game.description(),
                                    Style::default().fg(Color::Gray),
                                )),
                            ])
                        })
                        .collect();

                    let list = List::new(items)
                        .block(block)
                        .highlight_style(
                            Style::default()
                                .bg(Color::Blue)
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )
                        .highlight_symbol(">> ");

                    frame.render_stateful_widget(list, size, &mut self.get_list_state());
                })?;
                self.handle_events()?;
            }
            if self.quit {
                break;
            }

            println!(
                "[DEBUG] Launching game: {}",
                self.selectable_games[self.selected_index as usize].name()
            );

            let game = &mut self.selectable_games[self.selected_index as usize];
            let result = game.run(terminal);

            if result.is_err() {
                println!("[DEBUG] Game returned error, quitting...");
                self.quit = true;
            } else {
                println!("[DEBUG] Returned to menu.");
                self.in_game = false;
            }
        }
        println!("[DEBUG] Menu exited.");
        Ok(())
    }

    fn get_list_state(&self) -> ListState {
        let mut state = ListState::default();
        state.select(Some(self.selected_index as usize));
        state
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.in_game = true;
                self.quit = true;
            }
            KeyCode::Char('s') | KeyCode::Down => {
                if self.selected_index + 1 < self.selectable_games.len() as u32 {
                    self.selected_index += 1;
                }
            }
            KeyCode::Char('w') | KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Enter => {
                self.in_game = true;
            }
            _ => {}
        }
    }
}
