use super::Node;
use crate::geo::traits::Distance;

#[derive(Copy, Clone)]
pub struct Edge {
    pub from: i64,
    pub to: i64,
    pub distance: f64,
}

impl Edge {
    pub fn from_node_elements(from: &Node, to: &Node) -> Edge {
        let distance = from.distance(to);
        Edge { from: from.id, to: to.id, distance }
    }

    pub fn reversed(&self) -> Edge {
        Edge { from: self.to, to: self.from, distance: self.distance }
    }
}
