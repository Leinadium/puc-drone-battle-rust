mod logic;
mod query;
mod update;
mod node;
mod path;

use crate::api::map::path::Path;
use crate::api::enums::PlayerDirection;
use crate::Config;

use std::time::{Duration, SystemTime};
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct Coord {
    x: u8,
    y: u8
}

impl Coord {
    pub fn add(&mut self, c: &Coord) {
        self.x += c.x;
        self.y += c.y;
    }

    pub fn next(&self, dir: &PlayerDirection) -> Coord {
        match dir {
            PlayerDirection::NORTH => Coord { x: self.x, y: self.y - 1 },
            PlayerDirection::EAST => Coord { x: self.x + 1, y: self.y },
            PlayerDirection::SOUTH => Coord { x: self.x, y: self.y + 1 },
            PlayerDirection::WEST => Coord { x: self.x - 1, y: self.y },
        }
    }
}


#[derive(PartialEq, Debug, Clone)]
pub enum Position {
    SAFE,
    EMPTY,
    DANGER,
    UNKNOWN,
    WALL,
    GOLD,
    POWERUP
}

pub const MAP_WIDTH: u8 = 59;
pub const MAP_HEIGHT: u8 = 34;


pub struct Field {
    pub map: HashMap<Coord, Position>,
    pub gold_positions: HashMap<Coord, Duration>,
    pub powerup_positions: HashMap<Coord, Duration>,
    pub safe_positions: HashMap<Coord, bool>,
    pub unsafe_positions: HashMap<Coord, i32>,
    pub config: Config,
    pub spawn: Option<Coord>,

}

impl Field {
    pub fn new(config: &Config) -> Field {
        Field {
            map: HashMap::new(),
            gold_positions: HashMap::new(),
            powerup_positions: HashMap::new(),
            safe_positions: HashMap::new(),
            unsafe_positions: HashMap::new(),
            spawn: None ,
            config: config.clone()
        }
    }

    pub fn set_spawn(&mut self, c: &Coord) {
        self.spawn = Some(c.clone());
    }

    pub fn restart(&mut self) {
        self.map.clear();
        self.gold_positions.clear();
        self.powerup_positions.clear();
        self.safe_positions.clear();
        self.unsafe_positions.clear();
    }
}

impl Coord {
    pub fn manhattan(&self, c2: &Coord) -> u8 {
        ((self.x as i16 - c2.x as i16).abs() + (self.y as i16 - c2.y as i16).abs()) as u8
    }

    pub fn coords_5x2_sides(&self, dir: PlayerDirection) -> Vec<Coord> {
        match dir {
            PlayerDirection::NORTH | PlayerDirection::SOUTH => vec![
                Coord { x: self.x-2, y: self.y-2 }, Coord { x: self.x-1, y: self.y-2 },
                Coord { x: self.x+2, y: self.y-2 }, Coord { x: self.x+1, y: self.y-2 },
                Coord { x: self.x-2, y: self.y-1 }, Coord { x: self.x-1, y: self.y-1 },
                Coord { x: self.x+2, y: self.y-1 }, Coord { x: self.x+1, y: self.y-1 },
                Coord { x: self.x-2, y: self.y }, Coord { x: self.x-1, y: self.y },
                Coord { x: self.x+2, y: self.y }, Coord { x: self.x+1, y: self.y },
                Coord { x: self.x-2, y: self.y+1 }, Coord { x: self.x-1, y: self.y+1 },
                Coord { x: self.x+2, y: self.y+1 }, Coord { x: self.x+1, y: self.y+1 },
                Coord { x: self.x-2, y: self.y+2 }, Coord { x: self.x-1, y: self.y+2 },
                Coord { x: self.x+2, y: self.y+2 }, Coord { x: self.x+1, y: self.y+2 },
            ],
            _ => vec![
                Coord {x: self.x-2, y: self.y-2}, Coord {x: self.x-2, y: self.y-1},
                Coord {x: self.x-2, y: self.y+2}, Coord {x: self.x-2, y: self.y+1},

                Coord {x: self.x-1, y: self.y-2}, Coord {x: self.x-1, y: self.y-1},
                Coord {x: self.x-1, y: self.y+2}, Coord {x: self.x-1, y: self.y+1},

                Coord {x: self.x, y: self.y-2}, Coord {x: self.x, y: self.y-1},
                Coord {x: self.x, y: self.y+2}, Coord {x: self.x, y: self.y+1},

                Coord {x: self.x+1, y: self.y-2}, Coord {x: self.x-2, y: self.y-1},
                Coord {x: self.x+1, y: self.y+2}, Coord {x: self.x-2, y: self.y+1},

                Coord {x: self.x+2, y: self.y-2}, Coord {x: self.x-2, y: self.y-1},
                Coord {x: self.x+2, y: self.y+2}, Coord {x: self.x-2, y: self.y+1},
            ]
        }
    }
}