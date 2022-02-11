use std::fmt::{self, Debug, Formatter, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    FRONT,
    BACK,
    LEFT,
    RIGHT,
    GET,
    SHOOT,
    NOTHING
}


#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum ServerState {
    READY,
    GAME,
    DEAD,
    GAMEOVER
}

impl ServerState {
    pub fn from_str(st: &str) -> ServerState {
        match st.to_lowercase().as_str() {
            "ready" => ServerState::READY,
            "game" => ServerState::GAME,
            "dead" => ServerState::DEAD,
            "gameover" => ServerState::GAMEOVER,
            _ => ServerState::READY
        }
    }
}

impl fmt::Display for ServerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ServerState::READY => write!(f, "READY"),
            ServerState::DEAD => write!(f, "DEAD"),
            ServerState::GAME => write!(f, "GAME"),
            ServerState::GAMEOVER => write!(f, "GAME OVER")
        }
    }
}
