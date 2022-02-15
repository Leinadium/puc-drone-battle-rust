use crate::api::bot::BotData;
use crate::api::enums::{Action, PlayerDirection};
use crate::api::map::{Field, Position};
use crate::api::map::path::Path;
use crate::api::structs::LastObservation;
use crate::api::config::Config;
use crate::api::map::{query, update, logic, Coord};
use crate::api::map::update::SetType;

pub struct AI {
    // field
    pub field: Field,

    // base variables
    current_state: BotState,
    current_action: Action,

    // advanced variables
    ticks_running: i8,
    ticks_attacking: i8,
    map_changed: bool,
    c_buffer: Coord,
    going_to_powerup: bool,
    buffer_path: Option<Path>,

    // for exploration
    previous_state: BotState,
    current_path: Option<Path>,
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
            c_buffer: Coord { x: -1, y: -1},
            going_to_powerup: false,
            previous_state: BotState::NONE,
            current_path: None,
            buffer_path: None,
        }
    }

    pub fn think (&mut self, bot: BotData) -> Action {

        // probably dead, skipping
        if bot.get_energy() == 0 { return Action::NOTHING; }

        self.update_field(&bot);
        // updating states
        self.previous_state = self.current_state.clone();
        self.current_state = self.generate_state(&bot);

        // update internal variables and structures
        match self.current_state {
            BotState::RUN => self.do_run(&bot),
            BotState::ATTACK => self.do_attack(),
            BotState::COLLECT => self.do_collect(&bot),
            BotState::EXPLORE => self.do_explore(&bot),
            BotState::RECHARGE => self.do_recharge(&bot),
            _ => {}
        }

        // updating another internal variables and structures
        if self.current_state != BotState::ATTACK { self.ticks_attacking = 0; }
        self.c_buffer = Coord {x: bot.get_x(), y: bot.get_y() };

        println!("[AI] safe_positions: {}", self.field.safe_positions.len());
        println!("[AI] map_changed: {:?} | previous_state: {:?}", &self.map_changed, &self.previous_state);
        println!("[AI] path: {:?}", &self.current_path);
        println!("[AI] current_state: {:?} | current_action: {:?}", &self.current_state, &self.current_action);

        self.current_action.clone()
    }

    fn update_field(&mut self, bot: &BotData) {
        let dir: PlayerDirection = bot.get_dir();
        let c: Coord = Coord { x: bot.get_x(), y: bot.get_y() };
        let mut is_empty: bool = true;
        let mut is_danger: bool = false;
        let mut is_wall: bool = false;
        let o: LastObservation = bot.get_last_observation();
        self.map_changed = false;
        self.going_to_powerup = false;

        let f_mut: &mut Field = &mut self.field;

        // updates spawn if needed
        if f_mut.spawn.is_none() && bot.get_energy() > 0 {
            f_mut.set_spawn(&c);
        }

        // if the bot for some reason walks into a flash and teleports, update that position as a flash
        if ((c.x - self.c_buffer.x) + (c.y - self.c_buffer.y)) > 3 {
            update::set(f_mut, self.c_buffer.clone(), Position::DANGER, true);
            self.map_changed = true;
        }

        // if the bot took some damage, set some positions as unsafe so he can get out of there
        if o.is_damage {
            let unsafe_positions = vec![
                c.clone(), Coord { x: c.x - 1, y: c.y }, Coord { x: c.x - 2, y: c.y },
                Coord {x: c.x + 1, y: c.y}, Coord {x: c.x + 2, y: c.y},
                Coord {x: c.x, y: c.y - 1, }, Coord {x: c.x, y: c.y - 2},
                Coord {x: c.x, y: c.y + 1, }, Coord {x: c.x, y: c.y + 2}
            ];
            for up in unsafe_positions {
                update::set_unsafe(f_mut, up);
            }
        }

        // FLASH | HOLE
        if o.is_flash || o.is_breeze {
            update::set_custom(f_mut, &c, SetType::AROUND, None, Position::DANGER);
            is_danger = true;
            self.map_changed = true;
        }

        // WALL
        if o.is_blocked {
            if self.current_action == Action::FRONT {
                update::set_custom(f_mut, &c, SetType::FRONT, Some(dir.clone()), Position::WALL);
            } else {
                update::set_custom(f_mut, &c, SetType::BACK, Some(dir.clone()), Position::WALL);
            }
            is_wall = true;
            self.map_changed = true;
        }

        // POWERUP
        if o.is_powerup {
            update::remove_safe(f_mut, &c);
            update::set(f_mut, c.clone(), Position::POWERUP, false);
            update::set_powerup(f_mut, c.clone());
            is_empty = false;
        }

        // GOLD
        if o.is_treasure {
            update::remove_safe(f_mut, &c);
            update::set(f_mut, c.clone(), Position::GOLD, false);
            update::set_gold(f_mut, c.clone());
            is_empty = false;
        }

        // NOT DANGER
        if !is_danger {
            update::set_custom(f_mut, &c, SetType::AROUND, None, Position::SAFE);
            if !is_wall {
                if self.current_action == Action::FRONT {
                    update::set_custom(f_mut, &c, SetType::FRONT, Some(dir.clone()), Position::SAFE)
                } else {
                    update::set_custom(f_mut, &c, SetType::BACK, Some(dir.clone()), Position::SAFE)
                }
            }
        }

        // EMPTY
        if is_empty {
            update::remove_safe(f_mut, &c);
            update::set(f_mut, c.clone(), Position::EMPTY, false);
            logic::should_something_be_here(f_mut, &c);
        }
    }

    fn generate_state(&mut self, bot: &BotData) -> BotState {
        let o: LastObservation = bot.get_last_observation();
        let e: i32 = bot.get_energy();
        let c: Coord = Coord { x: bot.get_x(), y: bot.get_y() };
        let dir: PlayerDirection = bot.get_dir();
        let f: &Field = &self.field;

        // if there is gold, collect it right now
        if o.is_treasure { return BotState::COLLECT }

        // if there is a powerup and not full health, get it too
        if o.is_powerup && e <= 70 { return BotState::RECHARGE }

        // it has to run at least 5 times
        if self.ticks_running > 0 {
            self.ticks_running -= 1;
            return BotState::RUN;
        }

        // checking ATTACK
        if o.is_enemy_front &&
            self.ticks_attacking < 10 &&
            e > 30
            && !query::has_wall_front(f, &c, &dir, o.distance_enemy_front) {

            return BotState::ATTACK
        }

        // checking RUN
        if (o.is_damage && !o.is_enemy_front) || (
                (o.is_enemy_front || o.is_steps) && e < 30) {
            self.ticks_running = 5;
            return BotState::RUN;
        }

        // checking RECHARGE
        if e <= 80 { return BotState::RECHARGE }

        // checking collect
        let path_to_gold = query::has_gold_to_collect(f, &c, &dir);
        if query::has_gold(f) && path_to_gold.is_some() {
            self.buffer_path = path_to_gold;
            return BotState::COLLECT;
        }
        // checking explore
        BotState::EXPLORE
    }

    fn do_attack(&mut self) {
        self.ticks_attacking += 1;
        self.current_action = Action::SHOOT;
    }

    fn do_run(&mut self, bot: &BotData) {
        let o: LastObservation = bot.get_last_observation();
        let dir: PlayerDirection = bot.get_dir();
        let c: Coord = Coord {x: bot.get_x(), y: bot.get_y() };
        let f: &Field = &self.field;

        // keep running ?
        if let Some(p) = &self.current_path {
            if self.previous_state == BotState::RUN && self.ticks_running > 0 && p.size > 1 {
                self.current_path.as_mut().unwrap().pop_first_action();
                self.current_action =  self.current_path.as_ref().unwrap().get_first();
                return
            }
        }

        // second case
        if o.is_breeze {
            self.current_action = Action::LEFT;
            return
        }

        // first and third case
        let mut area: Vec<Coord> = c.coords_5x2_sides(&dir);
        area.retain(|c| {
            query::get(f, c) != Position::WALL && query::get(f, c) != Position::DANGER
        });

        let temp_path = logic::best_of_paths(f, &c, &dir, area, false);

        if let Some(tp) = temp_path {
            self.current_path = Some(tp);
            self.current_action = self.current_path.as_ref().unwrap().get_first();
        } else {
            self.do_attack();       // cannot run, try to at least fight back
        }
    }

    fn do_collect(&mut self, bot: &BotData) {
        if bot.get_last_observation().is_treasure {
            self.current_action = Action::GET;
            return;
        }

        // buffering from last move
        if let Some(cp) = &self.current_path {
            if self.previous_state == BotState::COLLECT && !self.map_changed && cp.size > 1 {
                self.current_path.as_mut().unwrap().pop_first_action();
                self.current_action = self.current_path.as_ref().unwrap().get_first();
                return;
            }
        }

        // geting from buffer, from has_gold_to_collect
        if let Some(bp) = &self.buffer_path {
            self.current_action = bp.get_first();
        } else {
            self.current_action = Action::NOTHING;  // error?
        }
    }

    fn do_recharge(&mut self, bot: &BotData) {
        let c: Coord = Coord { x: bot.get_x(), y: bot.get_y() };
        let dir: PlayerDirection = bot.get_dir();
        let f: &Field = &self.field;

        if bot.get_last_observation().is_powerup {
            self.current_action = Action::GET;
            return;
        }

        // checking for a powerup to run to
        let path_to_powerup_to_collect = query::has_powerup_to_collect(f, &c, &dir);
        if path_to_powerup_to_collect.is_some() && !self.going_to_powerup {
            self.map_changed = true;    // force to reload the buffer (?)
        }

        // using cached moves
        if let Some(cp) = &self.current_path {
            if self.previous_state == BotState::RECHARGE && !self.map_changed && cp.size > 1 {
                self.current_path.as_mut().unwrap().pop_first_action();
                self.current_action = self.current_path.as_ref().unwrap().get_first();
                return;
            }
        }

        // going to some powerup that is/will be ready
        if let Some(pc) = path_to_powerup_to_collect {
            self.going_to_powerup = true;
            self.current_path = Some(pc);
            self.current_action = self.current_path.as_ref().unwrap().get_first();
            return;
        }

        // searching for some powerup close by to explore
        if query::has_powerup(f) {
            if let Some(c_powerup) = logic::closest_powerup(f, &c, &dir) {
                if let Some(dest) = logic::best_block_using_midpoint(f, &c, &dir, &c_powerup) {
                    self.current_action = dest.get_first();
                    self.going_to_powerup = false;
                } else {
                    println!("AI [ERROR]: cannot find close block to explore while in need to recharge");
                    self.do_explore(bot);
                }
            } else {
                println!("AI [ERROR]: recharge bug");
                self.do_explore(bot);
            }
        }
        else {
            println!("AI [INFO]: while recharging, no powerup to collect or error");
            self.do_explore(bot);
        }
    }

    fn do_explore(&mut self, bot: &BotData) {
        // buffering from last move
        if let Some(cp) = &self.current_path {
            if self.previous_state == BotState::EXPLORE && !self.map_changed && cp.size > 1 {
                println!("[AI]: (explore) buffering from last move");
                self.current_path.as_mut().unwrap().pop_first_action();
                self.current_action = self.current_path.as_ref().unwrap().get_first();
                return;
            }
        }
        let f: &Field = &self.field;
        let c: Coord = Coord { x: bot.get_x(), y: bot.get_y() };
        let dir: PlayerDirection = bot.get_dir();


        let midpoint: Coord = logic::gold_midpoint(f);
        println!("[AI]: midpoint: ({}, {})", midpoint.x, midpoint.y);

        if let Some(p) = logic::best_block_using_midpoint(f, &c, &dir, &midpoint) {
            self.current_action = p.get_first();
            self.current_path = Some(p);

        } else {
            self.current_action = Action::NOTHING;
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
enum BotState {
    RUN,
    ATTACK,
    COLLECT,
    EXPLORE,
    RECHARGE,
    NONE
}