mod number_memory;
mod reaction;
mod r#trait;
mod utils;
mod verbal_memory;

use crate::games::number_memory::NumberMemory;
use crate::games::verbal_memory::VerbalMemory;
pub use reaction::ReactionGame;
pub use r#trait::Game;

pub fn get_all_games() -> Vec<Box<dyn Game>> {
    vec![
        Box::new(ReactionGame::new()),
        Box::new(VerbalMemory::new()),
        Box::new(NumberMemory::new()),
    ]
}
