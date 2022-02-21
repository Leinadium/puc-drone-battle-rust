mod api;

use crate::api::config::Config;
use crate::api::bot::Bot;
use crate::api::graphics::Graphics;

use std::env::args;

fn main() {
    // reading args
    let args: Vec<String> = args().collect();
    let config;
    if args.len() == 2 {
        let path = args.get(1).unwrap().clone();
        // loading config from file
        config = match Config::load(path) {
            Ok(c) => c,
            Err(e) => {
                println!("Error loading from config file: {}", e);
                println!("Using default configuration");
                Config::default()
            }
        }
    } else {
        println!("Using default configuration");
        config = Config::default();
    }



    // running the bot
    let graphics = match config.graphics {
        true => Graphics::new(config.name.clone()),
        false => None
    };
    println!("Graphics is set to {}", graphics.is_some());

    let mut bot = Bot::new(config, graphics);
    bot.run();

    bot.exit();
    println!("---- CLOSING ----");
}
