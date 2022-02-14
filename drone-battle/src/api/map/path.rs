use crate::api::enums::{Action, PlayerDirection};
use crate::api::map::Coord;
use crate::api::map::node::Node;

#[derive(Clone)]
pub struct Path {
    pub actions: Vec<Action>,
    pub size: usize,
    pub dest: Coord
}

impl Path {
    pub fn pop_first_action(&mut self) {
        self.actions.remove(0);
        self.size -= 1;
    }

    pub fn from_nodes(nodes: Vec<Node>) -> Option<Path> {
        let mut v: Vec<Action> = Vec::new();
        let mut previous: Option<&Node> = None;

        // getting the actions
        for node in nodes.iter() {
            let p = match previous {
                None => { previous = Some(node); continue }
                Some(n) => n
            };

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

        Some(Path {
            actions: v,
            size: v.len(),
            dest: nodes.last()?.coord.clone()
        })
    }
}