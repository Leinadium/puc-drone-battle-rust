use std::collections::HashMap;
use std::time::Duration;
use crate::api::enums::PlayerDirection;
use crate::api::map::{Position, Field, Coord, MAP_HEIGHT, MAP_WIDTH};
use crate::api::map::path::Path;
use crate::api::map::logic;

pub fn get(f: &Field, c: &Coord) -> Position {
    if c.x < 0 || c.y < 0 || c.x >= MAP_WIDTH || c.y >= MAP_HEIGHT { Position::WALL }
    else { f.map.get(c).unwrap_or(&Position::UNKNOWN) }
}

pub fn is_unsafe(f: &Field, c: &Coord) -> bool { f.unsafe_positions.contains_key(c) }

pub fn is_safe(f: &Field, c: &Coord) -> bool { f.safe_positions.contains_key(c) }

pub fn has_gold(f: &Field) -> bool { !f.gold_positions.is_empty() }

pub fn has_powerup(f: &Field) -> bool { !f.powerup_positions.is_empty() }

pub fn has_gold_to_collect(f: &Field, c: &Coord, dir: PlayerDirection) -> Option<Path> {
    has_something_to_collect(f, c, dir, &f.gold_positions)
}

pub fn has_powerup_to_collect(f: &Field, c: &Coord, dir: PlayerDirection) -> Option<Path> {
    has_something_to_collect(f, c, dir, &f.powerup_positions)
}

fn has_something_to_collect(
    f: &Field, current_coord: &Coord, dir: PlayerDirection, hm: &HashMap<Coord, Duration>
) -> Option<Path> {
    let mut time_to_born: Duration;
    let mut best_path: Option<Path> = None;
    let mut current_path: Option<Path>;

    for (something_coord, time) in hm.iter() {
        time_to_born = (f.config.spawn_timer - *time);
        current_path = logic::a_star(f,current_coord, &dir, &something_coord);
        match &current_path {
            None => continue,           // no path to place
            Some(cp) => {
                if time_to_born < cp.size * f.config.normal_timer {     // it will spawn in time
                    match &best_path {
                        // check best path first
                        Some(bp) => { if bp.size > cp.size { best_path = current_path } },
                        // no previous best path
                        None => { best_path = current_path }
                    }
                }
            }
        }
    }
    best_path
}

pub fn has_wall_front(f: &Field, c: &Coord, dir: PlayerDirection, q: u8) -> bool {
    let mut coords_to_check : Vec<Coord> = Vec::new();
    match dir {
        PlayerDirection::NORTH {
            for i in 1..q { coords_to_check.push( Coord { x: c.x, y: c.y - i } ) }
        },
        PlayerDirection::SOUTH {
            for i in 1..q { coords_to_check.push( Coord { x: c.x, y: c.y + i } ) }
        },
        PlayerDirection::EAST {
            for i in 1..q { coords_to_check.push( Coord { x: c.x + i, y: c.y } ) }
        },
        PlayerDirection::WEST {
            for i in 1..q { coords_to_check.push( Coord { x: c.x - i, y: c.y } ) }
        },
    }

    // check if any of these positions has a wall
    coords_to_check.iter().any(|c| get(f, c) == Position::WALL)
}