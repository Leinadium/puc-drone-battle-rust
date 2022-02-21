use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub id: String,
    pub bot: BotInfo,
    pub field: FieldInfo
}


#[derive(Serialize, Deserialize)]
pub struct BotInfo {
    pub x: i16,
    pub y: i16,
    pub dir: String,
    pub energy: i32,
    pub score: i64,
    pub state: String
}

#[derive(Serialize, Deserialize)]
pub struct FieldInfo {
    pub map: Vec<(i16, i16, String)>,
    pub powerup: Vec<(i16, i16, i64)>,
    pub gold: Vec<(i16, i16, i64)>,
    pub midpoint: (i16, i16),
    pub current_path: Vec<(i16, i16)>
}

