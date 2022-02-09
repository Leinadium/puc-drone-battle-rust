use crate::api::structs::{
    ServerObservation, ServerStatus, ServerGameStatus,
    ServerPlayer, ServerScoreboard, ServerNotification,
    ServerPlayerNew, ServerPlayerLeft, ServerChangeName,
    ServerHit, ServerDamage
};

pub enum PlayerDirection {
    NORTH,
    EAST,
    SOUTH,
    WEST
}

impl PlayerDirection {
    pub fn from_str(dir: &str) -> PlayerDirection {
        match dir {
            "1" => PlayerDirection::NORTH,
            "2" => PlayerDirection::EAST,
            "3" => PlayerDirection::SOUTH,
            "4" => PlayerDirection::WEST,
            _ => PlayerDirection::NORTH
        }
    }
}

pub enum ServerState {
    READY,
    GAME,
    DEAD,
    GAMEOVER
}

impl ServerState {
    pub fn from_str(st: &str) -> ServerState {
        match st {
            "1" => ServerState::READY,
            "2" => ServerState::GAME,
            "3" => ServerState::DEAD,
            "4" => ServerState::GAMEOVER,
            _ => ServerState::READY
        }
    }
}

pub enum Observation {
    ENEMYFRONT,
    BLOCKED,
    STEPS,
    BREEZE,
    FLASH,
    TREASURE,
    POWERUP
}

