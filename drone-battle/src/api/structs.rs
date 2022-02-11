use serde::{Serialize, Deserialize};

use std::fmt;

use crate::api::enums::{
    PlayerDirection, ServerState,
};

#[derive(Debug, Clone)]
pub struct LastObservation {
    pub is_enemy_front: bool,
    pub is_blocked: bool,
    pub is_steps: bool,
    pub is_breeze: bool,
    pub is_flash: bool,
    pub is_treasure: bool,
    pub is_powerup: bool,
    pub is_damage: bool,
    pub is_hit: bool,
    pub distance_enemy_front: i32,

    pub has_read_hit: bool,       // because hit and damage observations are separate
    pub has_read_damage: bool,
}

impl LastObservation {
    pub fn new() -> LastObservation {
        LastObservation {
            is_enemy_front: false,
            is_blocked: false,
            is_steps: false,
            is_breeze: false,
            is_flash: false,
            is_treasure: false,
            is_powerup: false,
            is_damage: false,
            is_hit: false,
            distance_enemy_front: -1,
            has_read_damage: true,
            has_read_hit: true,
        }
    }

    pub fn reset(&mut self) {
        self.is_enemy_front = false;
        self.is_blocked = false;
        self.is_steps = false;
        self.is_breeze = false;
        self.is_flash = false;
        self.is_treasure = false;
        self.is_powerup = false;
        self.is_damage = false;
        self.is_hit = false;
        self.distance_enemy_front = -1;
        self.has_read_damage = true;
        self.has_read_hit = true;
    }
}

impl fmt::Display for LastObservation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s: String = "".to_string();
        if self.is_enemy_front {s.push_str("ENEMY|");}
        if self.is_breeze {s.push_str("BREEZE|");}
        if self.is_flash {s.push_str("FLASH|");}
        if self.is_powerup {s.push_str("POWERUP|");}
        if self.is_blocked {s.push_str("BLOCKED|");}
        if self.is_hit {s.push_str("HIT|");}
        if self.is_damage {s.push_str("DAMAGE|");}
        if self.is_steps {s.push_str("STEPS|");}

        write!(f, "Observation {{ {} }} ", s)

    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn from_str(_c: &str) -> Color {
        // TODO
        Color { r: 0, g: 0, b: 0, a: 0 }
    }

    pub fn to_string(&self) -> String {
        format!("{};{};{}", &self.r, &self.g, &self.b)
    }
}

#[derive(Debug, Clone)]
pub struct Scoreboard {
    pub name: String,
    pub connected: bool,
    pub score: i64,
    pub energy: i32,
    pub color: Color,
}

#[derive(Debug)]
pub struct ServerObservation {
    pub last_observation: LastObservation
}

#[derive(Debug)]
pub struct ServerStatus {
    pub x: i8,
    pub y: i8,
    pub dir: PlayerDirection,
    pub state: ServerState,
    pub score: i64,
    pub energy: i32
}

#[derive(Debug, Clone)]
pub struct ServerPlayer {
    pub node: i64,
    pub name: String,
    pub x: i8,
    pub y: i8,
    pub dir: PlayerDirection,
    pub state: ServerState,
    pub color: Color
}

#[derive(Debug)]
pub struct ServerGameStatus {
    pub status: ServerState,
    pub time: i64
}

#[derive(Debug, Clone)]
pub struct ServerScoreboard {
    pub scoreboards: Vec<Scoreboard>
}

#[derive(Debug)]
pub struct ServerNotification {
    pub notification: String
}

#[derive(Debug)]
pub struct ServerPlayerNew {
    pub player: String
}

#[derive(Debug)]
pub struct ServerPlayerLeft {
    pub player: String
}

#[derive(Debug)]
pub struct ServerChangeName {
    pub old_name: String,
    pub new_name: String
}

#[derive(Debug)]
pub struct ServerHit {
    pub target: String
}

#[derive(Debug)]
pub struct ServerDamage {
    pub shooter: String
}

