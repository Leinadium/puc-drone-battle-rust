use serde::{Serialize, Deserialize};

use std::fmt;

use crate::api::enums::{
    PlayerDirection, ServerState,
};

/// Struct containing the last observation received for a drone
#[derive(Debug, Clone)]
pub struct LastObservation {
    /// the bot is facing an enemy (1-10 blocks in distance)
    pub is_enemy_front: bool,
    /// the bot is facing a wall
    pub is_blocked: bool,
    /// the bot is near some enemy
    pub is_steps: bool,
    /// the bot is near some hole
    pub is_breeze: bool,
    /// the bot is near some flash
    pub is_flash: bool,
    /// the bot is on top of a treasure
    pub is_treasure: bool,
    /// the bot is on top of a powerup
    pub is_powerup: bool,
    /// the bot took some damage
    pub is_damage: bool,
    /// the bot hit other drone
    pub is_hit: bool,
    /// the distance between the bot and an enemy, if `is_enemy_front` is set
    pub distance_enemy_front: i16,

    /// if the bot has processed the `is_hit` observation
    pub has_read_hit: bool,       // because hit and damage observations are separate
    /// if the bot has processed the `is_damage` observation
    pub has_read_damage: bool,
}

impl LastObservation {
    /// Generates a new one, with all negative values
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

    /// Resets to default values. Is the same of generating a new object
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

/// Struct for the color
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Color {
    /// red
    pub r: u8,
    /// green
    pub g: u8,
    /// blue
    pub b: u8,
    /// alpha (transparency)
    pub a: u8,
}

impl Color {
    /// converts a color from a string
    pub fn from_str(_c: &str) -> Color {
        // THIS IMPLEMENTATION DOES NOT NEED TO KNOW WHAT COLOR THE BOT IS.
        // SO, IT WILL ALWAYS RETURN BLACK
        Color { r: 0, g: 0, b: 0, a: 0 }
    }

    /// converts a color to a String
    pub fn to_string(&self) -> String {
        format!("{};{};{}", &self.r, &self.g, &self.b)
    }
}

/// Contain scoreboard information of a single bot/drone
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

