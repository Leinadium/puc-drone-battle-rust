mod api;

use crate::api::config::Config;
use crate::api::bot::Bot;

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
    let mut bot = Bot::new(config, true);
    bot.run();
    bot.exit();
    println!("---- CLOSING ----");
}
