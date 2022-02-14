extern crate pathfinding;

use std::time::Duration;
use crate::api::enums::PlayerDirection;
use crate::api::map::{Field, Coord, Position};
use crate::api::map::{update, query};
use crate::api::map::path::Path;
use crate::api::map::node::Node;

use pathfinding::prelude::astar;


pub fn a_star(f: &Field, origin: &Coord, dir: &PlayerDirection, dest: &Coord) -> Option<Path> {
    // running a_star
    let p: Option<(Vec<Node>, f64)> = astar(
        &Node { coord: origin.clone(), dir: dir.clone() },
        |n: Node| n.neighbours(f),
        |n: Node| n.distance_to_goal(dest),
        |n: Node| n.coord == *dest
    );

    // constructing the path
    match p {
        None => None,
        Some(nodes) => Path::from_nodes(nodes.0)
    }
}


pub fn should_something_be_here(f: &mut Field, c: &Coord) {
    let has_gold: bool = f.gold_positions.contains_key(c);
    let has_powerup: bool = f.powerup_positions.contains_key(c);

    if !has_gold && !has_powerup { return }

    let time: &Duration;
    if has_gold { time = f.gold_positions.get(c).unwrap() }
    else { time = f.powerup_positions.get(c).unwrap() }

    // should be here ?
    if time > &f.config.spawn_timer {
        // update its time
        if has_gold { update::set_gold(f, c.clone()) }
        else { update::set_powerup(f, c.clone()) }
    }
}


pub fn gold_midpoint(f: &Field) -> Coord {
    if !query::has_gold(f) { return f.spawn.unwrap_or( Coord{ x:0, y: 0} ).clone(); }

    let mut sum = Coord {x: 0, y: 0};
    for c in f.gold_positions.keys() { sum.add(c); }

    let size = f.gold_positions.len();

    Coord { x: sum.x / size, y: sum.y / size }
}

pub fn best_block_using_midpoint(f: &Field, c_bot: &Coord, dir: &PlayerDirection, c_mid: &Coord) -> Coord {
    let mut smallest_dist = 100000;
    let mut ret = Coord {x: 0, y: 0};
    let mut d;
    let mut dist_mid;
    let mut dist_bot;

    for c_safe in f.safe_positions.keys() {
        // getting distance from position to midpoint
        let t: i32 = ((*c_safe.x - c_mid.x).pow(2) + (*c_safe.y - c_mid.y).pow(2));
        dist_mid = (2 * (t as f32).sqrt()) as i32;

        // getting distance from position to bot
        dist_bot = match a_star(f, c_bot, dir, c_safe) {
            Some(p) => p.size,
            None => continue,
        } as i32;

        d = dist_mid + dist_bot;
        if d < smallest_dist {
            ret = c_safe.clone();
            smallest_dist = d;
        }
    }
    ret
}