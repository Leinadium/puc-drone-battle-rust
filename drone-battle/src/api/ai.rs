use crate::api::bot::BotData;
use crate::api::enums::{Action};
use crate::api::config::Config;

pub struct AI {
    // base variables
    current_action: Action,
    // graphics
    _verbose: bool
}

impl AI {
    pub fn new(_config: &Config, verbose: bool) -> AI {
        AI {
            current_action: Action::NOTHING,
            _verbose: verbose
        }
    }

    pub fn think (&mut self, bot: BotData) -> Action {
        // probably dead, skipping
        if bot.get_energy() == 0 { return Action::NOTHING; }

        self.current_action = Action::random();
        self.current_action.clone()
    }

}