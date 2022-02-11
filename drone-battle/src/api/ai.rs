use crate::api::bot::Bot;
use crate::api::enums::Action;

use rand::{thread_rng, Rng};

pub struct AI {
    // to be filed
}

impl AI {
    /// Generates a new AI
    pub fn new() -> AI {
        AI {  }
    }

    /// Selects a random action to be made
    pub fn think_random(&self, _bot: &Bot) -> Action {
        Action::random()
    }

}


impl Action {
    fn random() -> Self {
        match thread_rng().gen_range(0..6) {
            0 => Action::FRONT,
            1 => Action::BACK,
            2 => Action::LEFT,
            3 => Action::RIGHT,
            4 => Action::GET,
            _ => Action::SHOOT
        }
    }
}

