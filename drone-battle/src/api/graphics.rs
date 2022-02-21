pub mod connection;
pub mod data;

use crate::api::{
    graphics::{
        data::{Data, BotInfo, FieldInfo},
        connection::Connection,
    },
    bot::BotData,
    ai::AI,
    map::{Field}
};

pub struct Graphics {
    connection: Connection,
    ident: String,
}

impl Graphics {
    pub fn new(identifier: String) -> Option<Graphics> {
        let mut g = Graphics {
            connection: Connection::new(identifier.clone()),
            ident: identifier
        };
        if let None = g.connection.connect() {
            println!("[GRAPHICS] could not connect to mqtt");
            return None
        } else {
            println!("[GRAPHICS] sucessful connection")
        }
        Some(g)
    }

    pub fn update(&mut self, bot: &BotData, ai: &AI, field: &Field) {
        let id = self.ident.clone();

        let botinfo = BotInfo {
            x: bot.get_x(),
            y: bot.get_y(),
            dir: bot.get_dir().to_string(),
            energy: bot.get_energy(),
            score: bot.get_score(),
            state: ai.current_state.to_string(),
        };

        let map = field.map.iter().map(|(c, p)| {
            (c.x, c.y, p.to_string())
        }).collect();
        let gold = field.gold_positions.iter().map(|(c, p)| {
            (c.x, c.y, p.as_millis() as i64)
        }).collect();
        let powerup = field.powerup_positions.iter().map(|(c, p)| {
            (c.x, c.y, p.as_millis() as i64)
        }).collect();
        let midpoint = (field.buffer_midpoint_coord.x, field.buffer_midpoint_coord.y);
        let current_path: Vec<(i16, i16)> = match &ai.current_path {
            Some(p) => p.coords.iter().map(|c| {(c.x, c.y)}).collect(),
            None => vec![]
        };

        let fieldinfo = FieldInfo {
            map,
            gold,
            powerup,
            midpoint,
            current_path
        };


        let data = Data {
            id,
            bot: botinfo,
            field: fieldinfo
        };
        // println!("[GRAPHICS] sending data");
        self.connection.send(data).unwrap_or_else(|| {
            println!("[GRAPHICS] could not send update");
        });
    }

    pub fn close(self) {
        self.connection.disconnect();
    }
}
