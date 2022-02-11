use serde::{Deserialize, Serialize};
use serde_json as json;

use crate::api::structs::Color;

use std::fs;
use std::io::Error;
use std::time::Duration;
use rand::seq::SliceRandom;


#[derive(Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub url: String,
    pub slow_timer: Duration,
    pub normal_timer: Duration,
    pub min_timer: Duration,
    pub default_color: Color,
    pub spawn_timer: Duration
}

impl Config {
    pub fn default() -> Config {
        Config {
            name: random_string(10),
            url: "atari.icad.puc-rio.br".to_string(),
            slow_timer: Duration::from_millis(1000),
            normal_timer: Duration::from_millis(200),
            min_timer: Duration::from_millis(200),
            default_color: Color {r: 23, g: 179, b: 132, a: 0 },
            spawn_timer: Duration::from_millis(15600)
        }
    }

    pub fn load(filename: String) -> Result<Config, Error> {
        // opening file
        let content = fs::read_to_string(filename)?;

        // autoparsing
        let c: Config = json::from_str(content.as_str())?;

        Ok(c)
    }
}

/// temp
fn random_string(size: usize) -> String {
    let charset: Vec<&str> = "0123456789abcdef".split("").collect();
    let sample: Vec<&str> = charset
        .choose_multiple(&mut rand::thread_rng(), size)
        .cloned()
        .collect();

    sample.join("")
}