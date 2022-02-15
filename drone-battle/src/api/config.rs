use serde::{Deserialize, Serialize};
use serde_json as json;

use crate::api::structs::Color;

use std::fs;
use std::io::Error;
use std::time::Duration;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use rand::Rng;

#[derive(Clone)]
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
    pub fn from_config_json(c: ConfigJSON) -> Config {
        let default_color = Color { r: 0, g: 0, b: 0, a: 0};
        Config {
            name: c.name,
            url: c.url,
            slow_timer: Duration::from_millis(c.slow_timer),
            normal_timer: Duration::from_millis(c.normal_timer),
            min_timer: Duration::from_millis(c.min_timer),
            default_color: Color {
                r: c.default_color.get("r").unwrap_or(&default_color.r).clone(),
                g: c.default_color.get("g").unwrap_or(&default_color.g).clone(),
                b: c.default_color.get("b").unwrap_or(&default_color.b).clone(),
                a: c.default_color.get("a").unwrap_or(&default_color.a).clone(),
            },
            spawn_timer: Duration::from_millis(c.spawn_timer),
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct ConfigJSON {
    pub name: String,
    pub url: String,
    pub slow_timer: u64,
    pub normal_timer: u64,
    pub min_timer: u64,
    pub default_color: HashMap<String, u8>,
    pub spawn_timer: u64
}


impl Config {
    pub fn default() -> Config {
        Config {
            name: random_string(10),
            url: "atari.icad.puc-rio.br".to_string(),
            slow_timer: Duration::from_millis(1000),
            normal_timer: Duration::from_millis(100),
            min_timer: Duration::from_millis(100),
            default_color: random_color(),
            spawn_timer: Duration::from_millis(15000)
        }
    }

    pub fn load(filename: String) -> Result<Config, Error> {
        // opening file
        let content = fs::read_to_string(filename)?;

        // autoparsing
        let c: ConfigJSON = json::from_str(content.as_str())?;

        Ok(Config::from_config_json(c))
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

fn random_color() -> Color {
    let mut rng = rand::thread_rng();
    Color {
        r: rng.gen_range(0..255),
        g: rng.gen_range(0..255),
        b: rng.gen_range(0..255),
        a: 0
    }
}