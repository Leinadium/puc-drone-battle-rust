use crate::api::bot::BotData;
use crate::api::enums::{Action};
use crate::api::config::Config;

/// Artificial Intelligence struct
/// 
/// Contains all necessary variables to generate the next current action for the bot
pub struct AI {
    // base variables
    /// Current action to be made
    current_action: Action,
    /// Verbose option
    _verbose: bool
}

impl AI {
    /// Generates an AI object from a config object
    pub fn new(_config: &Config, verbose: bool) -> AI {
        AI {
            current_action: Action::NOTHING,
            _verbose: verbose
        }
    }

    /// Receives the current bot data, and returns the `Action` to be made
    pub fn think (&mut self, bot: BotData) -> Action {
        // probably dead, skipping
        if bot.get_energy() == 0 { return Action::NOTHING; }

        // for now, do a random action
        self.current_action = Action::random();
        self.current_action.clone()     // return
    }

}