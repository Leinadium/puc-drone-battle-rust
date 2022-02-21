use crate::api::enums::{Action};
use crate::api::map::Coord;
use crate::api::map::node::Node;

#[derive(Clone, Debug)]
pub struct Path {
    pub actions: Vec<Action>,
    pub size: usize,
    pub dest: Coord,
    pub coords: Vec<Coord>,
}

impl Path {
    pub fn pop_first_action(&mut self) {
        let a = self.actions.remove(0);
        match a {
            Action::FRONT | Action::BACK => {self.coords.remove(0);},
            _ => {},
        }
        self.size -= 1;
    }

    pub fn get_first(&self) -> Action {
        self.actions.first().unwrap_or(&Action::NOTHING).clone()
    }

    pub fn from_nodes(nodes: Vec<Node>) -> Option<Path> {
        let mut v: Vec<Action> = Vec::new();
        let mut previous: Option<&Node> = None;
        let mut v_coords: Vec<Coord> = vec![];

        // getting the actions
        for node in nodes.iter() {
            let p = match previous {
                None => {
                    previous = Some(node);
                    v_coords.push(node.coord.clone());
                    continue
                }
                Some(n) => n
            };

            if node.coord != p.coord { v_coords.push(node.coord.clone()) }

            if p.coord.next(&p.dir) == node.coord {
                v.push(Action::FRONT)       // front?
            }
            else if p.coord.next(&p.dir.opposite()) == node.coord {
                v.push(Action::BACK)      // back ?
            }
            else if p.dir.left() == node.dir {       // left?
                v.push(Action::LEFT)
            }
            else if p.dir.right() == node.dir {      // right?
                v.push(Action::RIGHT)
            }

            previous = Some(node);        // update previous
        }

        let size = v.len();
        Some(Path {
            actions: v,
            size,
            dest: nodes.last()?.coord.clone(),
            coords: v_coords
        })
    }
}