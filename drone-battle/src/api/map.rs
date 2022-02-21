pub mod logic;
pub mod query;
pub mod update;
pub mod node;
pub mod path;

use crate::api::enums::PlayerDirection;
use crate::Config;

use std::time::Duration;
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Debug, Clone, PartialOrd, Ord)]
pub struct Coord {
    pub x: i16,
    pub y: i16
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

impl Position {
    pub fn to_string(&self) -> String {
        match self {
            Position::SAFE => "SAFE".to_string(),
            Position::EMPTY => "EMPTY".to_string(),
            Position::DANGER => "DANGER".to_string(),
            Position::UNKNOWN => "UNKNOWN".to_string(),
            Position::WALL => "WALL".to_string(),
            Position::GOLD => "GOLD".to_string(),
            Position::POWERUP => "POWERUP".to_string(),
        }
    }
}

pub const MAP_WIDTH: i16 = 59;
pub const MAP_HEIGHT: i16 = 34;


pub struct Field {
    pub map: HashMap<Coord, Position>,
    pub gold_positions: HashMap<Coord, Duration>,
    pub powerup_positions: HashMap<Coord, Duration>,
    pub safe_positions: HashMap<Coord, bool>,
    pub unsafe_positions: HashMap<Coord, i32>,
    pub config: Config,
    pub spawn: Option<Coord>,

    pub buffer_midpoint_size: usize,
    pub buffer_midpoint_coord: Coord,

}

impl Field {
    pub fn new(config: &Config) -> Field {
        Field {
            map: HashMap::new(),
            gold_positions: HashMap::new(),
            powerup_positions: HashMap::new(),
            safe_positions: HashMap::new(),
            unsafe_positions: HashMap::new(),
            spawn: None,
            config: config.clone(),
            buffer_midpoint_size: 0,
            buffer_midpoint_coord: Coord {x: 0, y: 0}
        }
    }

    pub fn set_spawn(&mut self, c: &Coord) {
        self.spawn = Some(c.clone());
        self.buffer_midpoint_coord = c.clone();
    }

    pub fn restart(&mut self) {
        self.map.clear();
        self.gold_positions.clear();
        self.powerup_positions.clear();
        self.safe_positions.clear();
        self.unsafe_positions.clear();
        self.spawn = None;
        self.buffer_midpoint_size = 0;
        self.buffer_midpoint_coord = Coord {x: 0, y: 0}
    }
}

impl Coord {
    pub fn manhattan(&self, c2: &Coord) -> u8 {
        ((self.x as i16 - c2.x as i16).abs() + (self.y as i16 - c2.y as i16).abs()) as u8
    }

    pub fn coords_5x2_sides(&self, dir: &PlayerDirection) -> Vec<Coord> {
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