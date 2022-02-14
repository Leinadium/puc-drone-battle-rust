use crate::api::enums::PlayerDirection;
use crate::api::map::{Coord, Field, Position};
use crate::api::map::query;

use std::collections::HashMap;


#[derive(Clone, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
pub struct Node {
    pub coord: Coord,
    pub dir: PlayerDirection
}

impl Node {
    // using https://docs.rs/pathfinding/0.5.3/pathfinding/fn.astar.html
    pub fn distance_to_goal(&self, goal: &Coord) -> f64 {
        self.coord.manhattan(goal) as f64
    }

    pub fn neighbours(&self, f:&Field) -> Vec<(Node, f64)> {
        let find_cost = |c: &Coord, is_back: bool| -> f64 {
            let mut cost = 1.0;
            if is_back { cost += 1.0 }
            if query::is_safe(f, &self.coord) { cost *= 0.8 }
            if query::is_unsafe(f, &self.coord) { cost += 10.0 }
            cost
        };

        let front = self.coord.next(&self.dir);
        let back = self.coord.next(&self.dir.opposite());

        vec![
            ( Node { coord: front , dir: self.dir.clone()}, find_cost(&front, false) ),
            ( Node { coord: back , dir: self.dir.opposite()}, find_cost(&back, true) ),
            ( Node { coord: self.coord.clone(), dir: self.dir.left() }, 1.0),
            ( Node { coord: self.coord.clone(), dir: self.dir.right() }, 1.0),
        ]
    }
}
