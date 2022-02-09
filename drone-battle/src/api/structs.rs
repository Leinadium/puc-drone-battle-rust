use crate::api::enums::{
    PlayerDirection, ServerState,
};

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
    pub distance_enemy_front: i32
}

#[derive(Clone)]
pub struct Color {
    pub r: i8,
    pub g: i8,
    pub b: i8,
    pub a: i8,
}

impl Color {
    pub fn from_str(c: &str) -> Color {
        // TODO
        Color { r: 0, g: 0, b: 0, a: 0 }
    }

    pub fn to_string(&self) -> String {
        format!("{};{};{}", &self.r, &self.g, &self.b)
    }
}


pub struct Scoreboard {
    pub name: String,
    pub connected: bool,
    pub score: i64,
    pub energy: i8,
    pub color: Color,
}

pub struct ServerObservation {
    pub last_observation: LastObservation
}

pub struct ServerStatus {
    pub x: i8,
    pub y: i8,
    pub dir: PlayerDirection,
    pub state: ServerState,
    pub score: i64,
    pub energy: i8
}

pub struct ServerPlayer {
    pub node: i64,
    pub name: String,
    pub x: i8,
    pub y: i8,
    pub dir: PlayerDirection,
    pub state: ServerState,
    pub color: Color
}

pub struct ServerGameStatus {
    pub status: ServerState,
    pub time: i64
}

pub struct ServerScoreboard {
    pub scoreboards: Vec<Scoreboard>
}

pub struct ServerNotification {
    pub notification: String
}

pub struct ServerPlayerNew {
    pub player: String
}

pub struct ServerPlayerLeft {
    pub player: String
}

pub struct ServerChangeName {
    pub old_name: String,
    pub new_name: String
}

pub struct ServerHit {
    pub target: String
}

pub struct ServerDamage {
    pub shooter: String
}

