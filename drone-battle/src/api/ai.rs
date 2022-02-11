use crate::api::bot::Bot;
use crate::api::enums::Action;
use crate::api::map::{Field, Path};

use rand::{thread_rng, Rng};

pub struct AI {
    // field
    pub field: Field,

    // base variables
    current_state: BotState,
    current_action: Action,

    // advanced variables
    ticks_running: u8,
    ticks_attacking: u8,
    map_changed: bool,
    x_buffer: i8,
    y_buffer: i8,
    going_to_powerup: bool,

    // for exploration
    previous_state: Option<BotState>,
    current_path: Option<Path>
}

impl AI {
    pub fn new() -> AI {
        AI {
            field: Field::new(),
            current_state: BotState::EXPLORE,
            current_action: Action::NOTHING,
            ticks_running: 0,
            ticks_attacking: 0,
            map_changed: false,
            x_buffer: -1,
            y_buffer: -1,
            going_to_powerup: false,
            previous_state: None,
            current_path: None
        }
    }

    pub fn think_random(&self, _bot: &Bot) -> Action {
        Action::random()
    }


}

enum BotState {
    RUN,
    ATTACK,
    COLLECT,
    EXPLORE,
    RECHARGE,
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