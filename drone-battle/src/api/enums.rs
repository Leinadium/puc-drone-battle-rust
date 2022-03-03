use std::fmt::{self, Debug, Formatter, Result};
use rand::{thread_rng, Rng};


#[derive(Debug, Clone, PartialEq, )]
pub enum Action {
    FRONT,
    BACK,
    LEFT,
    RIGHT,
    GET,
    SHOOT,
    NOTHING
}

impl Action {
    pub fn to_str(&self) -> &str {
        match self {
            Action::FRONT => "FRONT",
            Action::BACK => "BACK",
            Action::LEFT => "LEFT",
            Action::RIGHT => "RIGHT",
            Action::GET => "GET",
            Action::SHOOT => "SHOOT",
            Action::NOTHING => "NOTHING"
        }
    }
    pub fn random() -> Self {
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


#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub enum PlayerDirection {
    NORTH,
    EAST,
    SOUTH,
    WEST
}

impl PlayerDirection {
    pub fn from_str(dir: &str) -> PlayerDirection {
        match dir {
            "north" => PlayerDirection::NORTH,
            "east" => PlayerDirection::EAST,
            "south" => PlayerDirection::SOUTH,
            "west" => PlayerDirection::WEST,
            _ => PlayerDirection::NORTH
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            PlayerDirection::NORTH => "NORTH".to_string(),
            PlayerDirection::EAST => "EAST".to_string(),
            PlayerDirection::SOUTH => "SOUTH".to_string(),
            PlayerDirection::WEST => "WEST".to_string(),

        }
    }

    pub fn opposite(&self) -> PlayerDirection {
        match self {
            PlayerDirection::NORTH => PlayerDirection::SOUTH,
            PlayerDirection::EAST => PlayerDirection::WEST,
            PlayerDirection::SOUTH => PlayerDirection::NORTH,
            PlayerDirection::WEST => PlayerDirection::EAST,
        }
    }

    pub fn right(&self) -> PlayerDirection {
        match self {
            PlayerDirection::NORTH => PlayerDirection::EAST,
            PlayerDirection::EAST => PlayerDirection::SOUTH,
            PlayerDirection::SOUTH => PlayerDirection::WEST,
            PlayerDirection::WEST => PlayerDirection::NORTH,
        }
    }

    pub fn left(&self) -> PlayerDirection { self.right().opposite() }
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
