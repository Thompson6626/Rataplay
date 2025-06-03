use crate::games::Game;
use crate::games::utils::line_with_color;
use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{Frame, Terminal};
use std::collections::HashSet;
use std::io;
use std::io::{Stdout};
use rand::prelude::{IndexedRandom, IteratorRandom};

enum GameState {
    Title,   // Initial screen
    Showing, // Showing words to player
    End,     // Game over
}
#[derive(Debug, PartialEq, Eq)]
enum Choice {
    SEEN,
    NEW,
}

pub struct VerbalMemory {
    state: GameState,
    words: Vec<String>,
    words_seen: HashSet<String>,
    word_shown: Option<String>,
    lives: u32,
    score: u32,
    choice: Choice,
    quit: bool,
}

impl Game for VerbalMemory {
    fn name(&self) -> &str {
        "🧠📝 Verbal Memory"
    }

    fn description(&self) -> &str {
        "Keep as many words in short term memory as possible"
    }

    fn handle_input(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.reset_game();

                match self.state {
                    GameState::Title => self.quit = true,
                    _ => self.state = GameState::Title,
                }
            }
            _ => {}
        }

        // Now handle state-specific actions.
        match self.state {
            GameState::Title => match key_event.code {
                KeyCode::Enter => {
                    self.assign_random_word_based_on_progress();
                    self.state = GameState::Showing;
                }
                _ => {}
            },
            GameState::Showing => {
                match key_event.code {
                    // Seen
                    KeyCode::Char('a') | KeyCode::Left => {
                        self.choice = Choice::SEEN;
                    }
                    // New
                    KeyCode::Char('d') | KeyCode::Right => {
                        self.choice = Choice::NEW;
                    }
                    KeyCode::Enter => {
                        // Default to false, so no points are reduced
                        let is_seen = self
                            .word_shown
                            .as_ref()
                            .map_or(false, |word| self.words_seen.contains(word));
                        let is_new = !is_seen;

                        // Adjust score and lives based on choice and correctness
                        if self.choice == Choice::SEEN {
                            if is_seen {
                                self.score += 1;
                            } else {
                                self.lives -= 1;
                            }
                        } else {
                            if is_new {
                                self.score += 1;
                            } else {
                                self.lives -= 1;
                            }
                            if let Some(word) = self.word_shown.as_ref() {
                                self.words_seen.insert(word.clone());
                            }
                        }

                        // Handle game over or progress
                        if self.lives <= 0 {
                            self.state = GameState::End;
                        } else {
                            self.assign_random_word_based_on_progress();
                        }
                    }
                    _ => {}
                }
            }
            GameState::End => match key_event.code {
                KeyCode::Enter => {
                    self.state = GameState::Title;
                    self.reset_game();
                }
                _ => {}
            },
        }
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        self.init_words_vec();

        while !self.quit {
            terminal
                .draw(|frame| match self.state {
                    GameState::Title => self.render_title_screen(frame),
                    GameState::Showing => self.render_on_game_screen(frame),
                    GameState::End => self.render_game_over_screen(frame),
                })
                .expect("Error while rendering game");

            self.handle_events()?;
        }

        self.quit_game();
        Ok(())
    }
}

impl VerbalMemory {
    pub fn new() -> Self {
        Self {
            state: GameState::Title,
            words: Vec::new(),
            words_seen: HashSet::new(),
            word_shown: None,
            lives: 3,
            score: 0,
            choice: Choice::SEEN,
            quit: false,
        }
    }

    fn render_title_screen(&self, frame: &mut Frame) {
        let lines = vec![
            line_with_color("Verbal Memory Test", Color::White)
                .style(Style::default().add_modifier(Modifier::BOLD)),
            line_with_color(
                "You will be shown words, one at a time. If you've seen a word during the test, click SEEN, If it's a new word, click NEW",
                Color::White,
            ),
        ];

        let size = frame.area();

        // Background fill
        let background = Block::default().style(Style::default().bg(Color::Cyan));
        frame.render_widget(background, size);

        // Layout to vertically center
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Min(3),
                Constraint::Percentage(40),
            ])
            .split(size);

        let paragraph: Paragraph<'_> = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(Block::default());

        frame.render_widget(paragraph, chunks[1]);
    }

    fn render_on_game_screen(&self, frame: &mut Frame) {
        let size = frame.area();

        // Background
        let background = Block::default().style(Style::default().bg(Color::Cyan));
        frame.render_widget(background, size);

        // Vertical layout
        let outer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Length(3), // Score + Lives
                Constraint::Length(3), // Word
                Constraint::Length(3), // Buttons
                Constraint::Percentage(30),
            ])
            .split(size);

        // Score and Lives
        let score_line = format!("Score: {}    Lives: {}", self.score, self.lives);

        let score_paragraph = Paragraph::new(score_line)
            .style(Style::default().fg(Color::White).bg(Color::Cyan))
            .alignment(Alignment::Center);
        frame.render_widget(score_paragraph, outer_chunks[1]);

        // Word shown
        let word_text = Paragraph::new(self.word_shown.as_deref().unwrap_or(""))
            .style(Style::default().fg(Color::White).bg(Color::Cyan))
            .alignment(Alignment::Center);
        frame.render_widget(word_text, outer_chunks[2]);

        // Buttons: centered horizontally
        let button_width = 10;
        let total_button_width = (button_width * 2) + 4; // spacing between + margin
        let x_offset = (size.width.saturating_sub(total_button_width)) / 2;

        let button_area = Rect {
            x: size.x + x_offset,
            y: outer_chunks[3].y,
            width: total_button_width,
            height: outer_chunks[3].height / 2,
        };

        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(button_width),
                Constraint::Length(4), // gap between
                Constraint::Length(button_width),
            ])
            .split(button_area);

        let (seen_style, new_style) = match self.choice {
            Choice::SEEN => (
                Style::default()
                    .fg(Color::Cyan)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(Color::White).bg(Color::Cyan),
            ),
            Choice::NEW => (
                Style::default().fg(Color::White).bg(Color::Cyan),
                Style::default()
                    .fg(Color::Cyan)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
        };

        let seen = Paragraph::new("Seen")
            .style(seen_style)
            .alignment(Alignment::Center);
        let new = Paragraph::new("New")
            .style(new_style)
            .alignment(Alignment::Center);

        frame.render_widget(seen, button_chunks[0]);
        frame.render_widget(new, button_chunks[2]);
    }

    fn render_game_over_screen(&self, frame: &mut Frame) {
        let lines = vec![
            line_with_color("Verbal Memory", Color::White),
            line_with_color(format!("{} words", self.score), Color::White)
                .style(Style::default().add_modifier(Modifier::BOLD)),
            line_with_color("Press to continue", Color::White),
        ];

        let size = frame.area();

        // Background fill
        let background = Block::default().style(Style::default().bg(Color::Cyan));
        frame.render_widget(background, size);

        // Layout to vertically center
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Min(3),
                Constraint::Percentage(40),
            ])
            .split(size);

        let paragraph: Paragraph<'_> = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(Block::default());

        frame.render_widget(paragraph, chunks[1]);
    }
    
    fn init_words_vec(&mut self) {
        if !self.words.is_empty() {
            return;
        }

        let file_content = include_str!("../../assets/palabras.txt");
        self.words = file_content
            .lines()
            .map(|line| line.trim().to_string())
            .collect();

        println!("Loaded {} words", self.words.len());
    }


    fn assign_random_word_based_on_progress(&mut self) {
        let mut rng = rand::rng();

        self.word_shown = if rng.random::<f64>() < 0.7 {
            // 70% chance: pick from words Vec
            self.words.choose(&mut rng).cloned()
        } else {
            // 30% chance: pick from words_seen HashSet
            self.words_seen.iter().choose(&mut rng).cloned()
        };
    }

    fn reset_game(&mut self) {
        self.clear_progress();
    }

    fn quit_game(&mut self) {
        self.clear_progress();
        self.words.clear();
        self.quit = false;
    }

    fn clear_progress(&mut self) {
        self.words_seen.clear();
        self.lives = 3;
        self.score = 0;
        self.word_shown = None;
    }
}
