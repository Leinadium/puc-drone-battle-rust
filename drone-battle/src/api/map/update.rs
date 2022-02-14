use crate::api::map::{Field, Position, Coord};
use crate::api::map::query;

use std::time::Duration;
use crate::api::enums::PlayerDirection;


pub fn set(f: &mut Field, c: Coord, p: Position, force: bool) {
    let current_position = query::get(f, &c);

    if force {
        match &p {
            Position::POWERUP => set_powerup(f, c.clone()),
            Position::GOLD => set_gold(f, c.clone()),
            _ => {}
        }
        f.map.insert(c, p);
        return
    }

    // check if it already knew
    if p == current_position { return; }

    // is spawn empty? so set it
    if p == Position::EMPTY || f.spawn.is_none() {
        f.set_spawn(&c);
    }

    match p {
        Position::DANGER => {
            if current_position == Position::UNKNOWN {
                f.map.insert(c.clone(), p)
            }
            return
        },
        Position::SAFE => {
            if current_position == Position::UNKNOWN || current_position == Position::DANGER {
                f.map.insert(c.clone(), p);
                set_safe(f, c.clone());
            }
            return
        },
        Position::EMPTY =>  {
            if current_position != Position::GOLD && current_position != Position::POWERUP {
                f.map.insert(c.clone(), p);
            }
            return
        },
        _ => {}
    }

    remove_safe(f, &c);
    f.map.insert(c, p);
}

pub fn set_gold(f: &mut Field, c: Coord) {
    f.gold_positions.insert(c, Duration::from_secs(0));
}

pub fn set_powerup(f: &mut Field, c: Coord) {
    f.powerup_positions.insert(c, Duration::from_secs(0));
}

pub fn set_safe(f: &mut Field, c: Coord) { f.safe_positions.insert(c, true); }

pub fn set_unsafe(f: &mut Field, c: Coord) { f.unsafe_positions.insert(c, 1); }

pub fn remove_safe(f: &mut Field, c: &Coord) { f.safe_positions.remove(c).ok(); }

pub fn set_custom(f: &mut Field, c: &Coord, set_type: SetType, dir: Option<PlayerDirection>, p: Position) {
    match set_type {
        SetType::AROUND => {
            set(f, Coord {x: c.x + 1, y: c.y }, p.clone(), false);
            set(f, Coord {x: c.x - 1, y: c.y }, p.clone(), false);
            set(f, Coord {x: c.x, y: c.y + 1 }, p.clone(), false);
            set(f, Coord {x: c.x, y: c.y - 1 }, p.clone(), false);
        },
        SetType::FRONT | SetType::BACK => {
            let mut d = dir.unwrap_or(PlayerDirection::NORTH);
            if set_type == SetType::BACK {  // inverting for a back insert
                d = match d {
                    PlayerDirection::NORTH => PlayerDirection::SOUTH,
                    PlayerDirection::WEST => PlayerDirection::EAST,
                    PlayerDirection::SOUTH => PlayerDirection::NORTH,
                    PlayerDirection::EAST => PlayerDirection::WEST
                }
            }
            match d {
                PlayerDirection::NORTH => set(f, Coord { x: c.x, y: c.y - 1 }, p, false),
                PlayerDirection::EAST  => set(f, Coord { x: c.x + 1, y: c.y }, p, false),
                PlayerDirection::SOUTH => set(f, Coord { x: c.x, y: c.y + 1 }, p, false),
                PlayerDirection::WEST  => set(f, Coord { x: c.x - 1, y: c.y }, p, false),
            }
        },
    }
}

pub fn do_tick(f: &mut Field, dur: Duration) {
    // updating all gold ticks
    for g in f.gold_positions.values_mut() { *g += dur; }
    // updating all powerup ticks
    for pw in f.powerup_positions.values_mut() { *pw += dur; }

    // updating all unsafe positions
    // removes if tick > 7, else += 1
    f.unsafe_positions.retain(|_, &t| t > 7);
    for up in f.unsafe_positions.values_mut() { *up += 1 }

}

pub enum SetType {
    FRONT,
    AROUND,
    BACK
}