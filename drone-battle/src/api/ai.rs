use crate::api::bot::Player;
use crate::api::enums::{Action, PlayerDirection, ServerState};
use crate::api::map::{Field, Path, Position};
use crate::api::structs::LastObservation;
use crate::api::config::Config;


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
    pub fn new(config: &Config) -> AI {
        AI {
            field: Field::new(config),
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

    pub fn think<T: Player> (&mut self, _bot: &T) -> Action {
        self.update_field(_bot);
        // updating states
        self.previous_state = Some(self.current_state.clone());
        self.current_state = self.generate_state(_bot);

        // update internal variables and structures
        match self.current_state {
            BotState::RUN => self.do_run(),
            BotState::ATTACK => self.do_attack(),
            BotState::COLLECT => self.do_collect(),
            BotState::EXPLORE => self.do_explore(),
            BotState::RECHARGE => self.do_explore()
        }

        // updating another internal variables and structures
        if self.current_state != BotState::ATTACK { self.ticks_attacking = 0; }
        self.x_buffer = _bot.get_x();
        self.y_buffer = _bot.get_y();

        self.current_action.clone()
    }

    fn update_field<T: Player> (&mut self, _bot: &T) {
        let _dir: PlayerDirection = _bot.get_dir();
        let x: i8 = _bot.get_x();
        let y: i8 = _bot.get_y();
        let mut _is_empty: bool = true;
        let mut is_danger: bool = false;
        let mut _is_wall: bool = false;
        let o: LastObservation = _bot.get_last_observation();
        self.map_changed = false;
        self.going_to_powerup = false;

        // if the bot for some reason walks into a flash and teleports, update that position as a flash
        if ((x - self.x_buffer).abs() + (y - self.y_buffer).abs()) > 3 {
            self.field.setForce(x_buffer, y_buffer, Position::DANGER);
            self.map_changed = true;
        }

        // if the bot took some damage, set some positions as unsafe so he can get out of there
        if o.is_damage {
            self.field.set_unsafe(&x, &y);
            self.field.set_unsafe(&(x - 1), &y);
            self.field.set_unsafe(&(x - 2), &y);

            self.field.set_unsafe(&(x + 1), &y);
            self.field.set_unsafe(&(x + 2), &y);

            self.field.set_unsafe(&x, &(y - 1));
            self.field.set_unsafe(&x, &(y - 2));

            self.field.set_unsafe(&x, &(y + 1));
            self.field.set_unsafe(&x, &(y + 2));
        }

        // FLASH | HOLE
        if o.is_flash || o.is_breeze {
            self.field.set_around(&x, &y, Position::DANGER);
            is_danger = true;
            self.map_changed = true;
        }

        // WALL
        if o.is_blocked {
            if self.current_action == Action::FRONT {

            }
        }

    }

    fn generate_state<T: Player>(&self, _bot: &T) -> BotState {
        // TODO: calcular
        BotState::EXPLORE
    }

    fn do_run(&mut self) { }

    fn do_attack(&mut self) { }

    fn do_collect(&mut self) { }

    fn do_explore(&mut self) { }

    fn do_recharge(&mut self) { }

    pub fn think_random<T: Player> (&self, _bot: &T) -> Action {
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