extern crate ordered_float;

use crate::api::enums::PlayerDirection;
use crate::api::map::{Coord, Field, Position};
use crate::api::map::query;

use ordered_float::OrderedFloat;


#[derive(Clone, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
pub struct Node {
    pub coord: Coord,
    pub dir: PlayerDirection
}

impl Node {
    // using https://docs.rs/pathfinding/0.5.3/pathfinding/fn.astar.html
    pub fn distance_to_goal(&self, goal: &Coord) -> OrderedFloat<f64> {
        OrderedFloat(self.coord.manhattan(goal) as f64)
    }

    pub fn neighbours(&self, f:&Field) -> Vec<(Node, OrderedFloat<f64>)> {
        let find_cost = |c: &Coord, is_back: bool| -> OrderedFloat<f64> {
            let mut cost = 1.0;
            if is_back { cost += 1.5 }
            if query::is_safe(f, &c) { cost *= 0.8 }
            if query::is_unsafe(f, &c) { cost += 10.0 }
            OrderedFloat(cost)
        };

        let front = self.coord.next(&self.dir);
        let back = self.coord.next(&self.dir.opposite());

        let mut ret: Vec<(Node, OrderedFloat<f64>)> = vec![
            ( Node { coord: front.clone() , dir: self.dir.clone()}, find_cost(&front, false) ),
            ( Node { coord: back.clone() , dir: self.dir.clone()}, find_cost(&back, true) ),
            ( Node { coord: self.coord.clone(), dir: self.dir.left() }, OrderedFloat(1.0)),
            ( Node { coord: self.coord.clone(), dir: self.dir.right() }, OrderedFloat(1.0)),
        ];
        ret.retain(|(n, _)| {
            match query::get(f, &n.coord) {
                Position::DANGER | Position::WALL | Position::UNKNOWN => false,
                _ => true
            }
        });
        ret
    }
}
