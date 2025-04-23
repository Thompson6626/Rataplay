mod reaction;

use crate::reaction::{ReactionGame, ui};
use crossterm::event::{KeyEventKind, poll};
use crossterm::{
    event,
    event::Event,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};
use std::time::Duration;
use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout: io::Stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut game = ReactionGame::new();

    loop {
        terminal.draw(|f| ui(f, &game))?;

        if poll(Duration::from_millis(30))? {
            fn handle_events(game: &mut ReactionGame) -> io::Result<()> {
                match event::read()? {
                    // it's important to check that the event is a key press event as
                    // crossterm also emits key release and repeat events on Windows.
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        game.handle_input(key_event);
                    }
                    _ => {}
                };
                Ok(())
            }

            handle_events(&mut game)?;
        }

        // Always update the game state
        game.update();
    }
}
