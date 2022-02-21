extern crate pathfinding;

use std::borrow::Borrow;
use std::time::Duration;
use crate::api::enums::PlayerDirection;
use crate::api::map::{Field, Coord};
use crate::api::map::{update, query};
use crate::api::map::path::Path;
use crate::api::map::node::Node;

use pathfinding::prelude::astar;
use ordered_float::OrderedFloat;


pub fn a_star(f: &Field, origin: &Coord, dir: &PlayerDirection, dest: &Coord) -> Option<Path> {
    // running a_star
    let p: Option<(Vec<Node>, OrderedFloat<f64>)> = astar(
        &Node { coord: origin.clone(), dir: dir.clone() },
        |n: &Node| n.neighbours(f),
        |n: &Node| n.distance_to_goal(dest),
        |n: &Node| n.coord == *dest
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

pub fn gold_midpoint(f: &mut Field) -> Coord {
    if !query::has_gold(f) { return f.spawn.as_ref().unwrap_or( &Coord{ x:0, y: 0} ).clone(); }

    // check buffer
    if f.gold_positions.len() == f.buffer_midpoint_size {
        return f.buffer_midpoint_coord.clone()
    }

    let mut sum = Coord {x: 0, y: 0};
    for c in f.gold_positions.keys() { sum.add(c); }
    let size = f.gold_positions.len() as i16;
    let ret = Coord { x: sum.x / size, y: sum.y / size };

    // update buffer
    f.buffer_midpoint_coord = ret.clone();
    f.buffer_midpoint_size = f.gold_positions.len();

    ret
}

pub fn best_block_using_midpoint(f: &Field, c_bot: &Coord, dir: &PlayerDirection, c_mid: &Coord) -> Option<Path> {
   let mut smallest_dist = 100000;
    let mut d;
    let mut dist_block_point;
    let mut path_ret: Option<Path> = None;
    let mut temp_path;

    for c_safe in f.safe_positions.keys() {
        // getting distance from position to midpoint
        let t: f64 = ((c_safe.x - c_mid.x).pow(2) + (c_safe.y - c_mid.y).pow(2)) as f64;
        dist_block_point = 2 * (t.sqrt() as i32);

        // getting distance from position to bot
        temp_path = match a_star(f, c_bot, dir, c_safe) {
            Some(p) => p,
            None => continue,
        };

        d = dist_block_point + (temp_path.size as i32);
        if d < smallest_dist {
            smallest_dist = d;
            path_ret = Some(temp_path);
        }
    }
    path_ret
}

pub fn best_of_paths(f: &Field, c: &Coord, dir: &PlayerDirection, coords: Vec<Coord>, smallest: bool) -> Option<Path> {
    let compare = if smallest { |c1, c2| { c1 < c2 } } else { |c1, c2| { c1 > c2 } };

    let mut temp_path: Option<Path> = None;
    let mut p;
    for coord in coords {
        // checking if there is a path

        let mut update: bool = true;
        p = match a_star(f, &c, &dir, &coord) {
            None => continue,
            Some(p) => p
        };
        if let Some(ref tp) = temp_path {
            if !compare(p.borrow().size, tp.size) { update = false; }
        }
        if update { temp_path = Some(p); }

    }
    temp_path
}

pub fn closest_powerup(f: &Field, c: &Coord, dir: &PlayerDirection) -> Option<Coord> {
    let v: Vec<Coord> = f.powerup_positions.keys().cloned().collect::<Vec<Coord>>();
    Some(best_of_paths(f, c, dir, v, true)?.dest)
}