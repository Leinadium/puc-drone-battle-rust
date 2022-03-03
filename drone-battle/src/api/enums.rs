use std::fmt::{self, Debug, Formatter, Result};
use rand::{thread_rng, Rng};


/// The action the bot/drone may take in each turn
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
    /// Converts into a stt
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
    /// Get a random action
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


/// All cardinal directions
#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub enum PlayerDirection {
    NORTH,
    EAST,
    SOUTH,
    WEST
}

impl PlayerDirection {
    /// Get a PlayerDirection object from a str representation
    pub fn from_str(dir: &str) -> PlayerDirection {
        match dir {
            "north" => PlayerDirection::NORTH,
            "east" => PlayerDirection::EAST,
            "south" => PlayerDirection::SOUTH,
            "west" => PlayerDirection::WEST,
            _ => PlayerDirection::NORTH
        }
    }

    /// Converts the object into a String
    pub fn to_string(&self) -> String {
        match self {
            PlayerDirection::NORTH => "NORTH".to_string(),
            PlayerDirection::EAST => "EAST".to_string(),
            PlayerDirection::SOUTH => "SOUTH".to_string(),
            PlayerDirection::WEST => "WEST".to_string(),

        }
    }

    /// Get the opposite cardinal direction
    pub fn opposite(&self) -> PlayerDirection {
        match self {
            PlayerDirection::NORTH => PlayerDirection::SOUTH,
            PlayerDirection::EAST => PlayerDirection::WEST,
            PlayerDirection::SOUTH => PlayerDirection::NORTH,
            PlayerDirection::WEST => PlayerDirection::EAST,
        }
    }

    /// Get the cardinal direction to the right (90 degrees clockwise)
    pub fn right(&self) -> PlayerDirection {
        match self {
            PlayerDirection::NORTH => PlayerDirection::EAST,
            PlayerDirection::EAST => PlayerDirection::SOUTH,
            PlayerDirection::SOUTH => PlayerDirection::WEST,
            PlayerDirection::WEST => PlayerDirection::NORTH,
        }
    }

    /// Get the cardinal direction to the left (90 degrees counterclockwise)
    pub fn left(&self) -> PlayerDirection { self.right().opposite() }
}

/// All possible Game states
#[derive(Debug, Clone, PartialEq)]
pub enum ServerState {
    /// Game is about to start
    READY,
    /// Game is running
    GAME,
    /// The bot is dead
    DEAD,
    /// Game is over
    GAMEOVER
}

impl ServerState {
    /// Creates an object from a str representation
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
