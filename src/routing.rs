use std::collections::{HashMap, HashSet};
use std::collections::binary_heap::BinaryHeap;
use std::cmp::{Eq, Ord, Ordering, PartialOrd};

use crate::geo::Coordinates;
use crate::geo::graph::Graph;
use crate::geo::traits::Distance;

struct MinHeapElement {
    id: i64,
    f_score: u64,
}

impl Eq for MinHeapElement {}

impl Ord for MinHeapElement {
    fn cmp(&self, other: &MinHeapElement) -> Ordering {
        other.f_score.partial_cmp(&self.f_score).unwrap_or(Ordering::Less)
    }
}

impl PartialOrd for MinHeapElement {
    fn partial_cmp(&self, other: &MinHeapElement) -> Option<Ordering> {
        other.f_score.partial_cmp(&self.f_score)
    }
}

impl PartialEq for MinHeapElement {
    fn eq(&self, other: &MinHeapElement) -> bool {
        self.id == other.id && self.f_score == other.f_score
    }
}

pub fn route(graph: &Graph, start_coords: &Coordinates, end_coords: &Coordinates) -> Vec<i64> {
    let start = graph.id_to_node.values().min_by_key(
        |node| { start_coords.distance(node) }
    ).unwrap().id;
    let end = graph.id_to_node.values().min_by_key(
        |node| { end_coords.distance(node) }
    ).unwrap().id;

    let mut closed_set: HashSet<i64> = HashSet::new();
    let mut g_score: HashMap<i64, u64> = HashMap::new();
    let mut open_set: BinaryHeap<MinHeapElement> = BinaryHeap::new();
    let mut father: HashMap<i64, i64> = HashMap::new();

    let mut route: Option<Vec<i64>> = None;

    let heuristic = |id| {
        let u = &graph.id_to_node[&id];
        let v = &graph.id_to_node[&end];
        u.distance(v)
    };

    let retrace = |id: i64, prevs: &HashMap<i64, i64>| {
        let mut path: Vec<i64> = vec![id];
        let mut current_id = id;

        while current_id != start {
            match prevs.get(&current_id) {
                Some(prev) => {
                    current_id = *prev;
                    path.push(current_id);
                },
                None => {
                    panic!("Cannot find path to start node from end node");
                }
            }
        }

        path.reverse();
        path
    };

    g_score.insert(start, 0);
    open_set.push(MinHeapElement { id: start, f_score: heuristic(start) });

    while !open_set.is_empty() {
        let current = open_set.pop().unwrap();
        closed_set.insert(current.id);

        if current.id == end {
            route = Some(retrace(current.id, &father));
        } else if let Some(edges) = graph.id_to_edges.get(&current.id) {
            let open_edges = edges
                .iter()
                .filter(|edge| !closed_set.contains(&edge.to));
            for edge in open_edges {
                let tentative_g_score = g_score[&current.id] + edge.distance;

                let mut update_scores =
                    |g_score: &mut HashMap<i64, u64>| {
                    g_score.insert(edge.to, tentative_g_score);
                    open_set.push(
                        MinHeapElement {
                            id: edge.to,
                            f_score: tentative_g_score + heuristic(edge.to),
                        }
                    );
                    father.insert(edge.to, current.id);
                };

                match g_score.get(&edge.to) {
                    Some(neighbor_g_score) => {
                        if tentative_g_score < *neighbor_g_score {
                            update_scores(&mut g_score);
                        }
                    },
                    None => {
                        update_scores(&mut g_score);
                    }
                }
            }
        }
    }

    route.expect("No route found")
}
