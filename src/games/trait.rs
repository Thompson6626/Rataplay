use crossterm::event;
use crossterm::event::{Event, KeyEvent, KeyEventKind};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;

pub trait Game {
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    // Default just reading keys
    fn handle_events(&mut self) -> io::Result<()> {
        // Just handling key presses
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_input(key_event)
            }
            _ => {}
        };
        Ok(())
    }
    fn handle_input(&mut self, key_event: KeyEvent);

    // Games can return to choose between terminating the whole game or just going back to menu
    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()>;
}
