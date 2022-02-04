use proc_macro::bridge::server::Server;

pub enum PlayerDirection {
    NORTH,
    EAST,
    SOUTH,
    WEST
}

pub fn to_player_direction(dir: &str) -> PlayerDirection {
    match dir {
        "1" => PlayerDirection::NORTH,
        "2" => PlayerDirection::EAST,
        "3" => PlayerDirection::SOUTH,
        "4" => PlayerDirection::WEST,
        _ => PlayerDirection::NORTH
    }
}

pub enum ServerState {
    READY,
    GAME,
    DEAD,
    GAMEOVER
}

pub fn to_server_state(st: &str) -> ServerState {
    match dir {
        "1" => ServerState::READY,
        "2" => ServerState::GAME,
        "3" => ServerState::DEAD,
        "4" => ServerState::GAMEOVER,
        _ => ServerState::READY
    }
}

pub enum RecvCommand {
    Observations(ServerObservation),
    Status(ServerStatus),
    Player(ServerPlayer),
    GameStatus(ServerGameStatus),
    Scoreboard(ServerScoreboard),
    Notification(ServerNotification),
    PlayerNew(ServerPlayerNew),
    PlayerLeft(ServerPlayerLeft),
    ChangeName(ServerChangeName),
    Hit(ServerHit),
    Damage(ServerDamage),
    Invalid
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

pub struct Color {
    pub r: i8,
    pub g: i8,
    pub b: i8,
    pub a: i8,
}

pub fn to_color(c: &str) -> Color {
    // TODO

    // let for_split: &[char] = &[',', ']'];
    // let p = c.split(for_split).collect()
    Color { r: 0, g: 0, b: 0, a: 0 }
}

pub struct Scoreboard {
    pub name: String,
    pub connected: bool,
    pub score: i64,
    pub energy: i8,
    pub color: Color,
}

pub struct ServerObservation {}

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

