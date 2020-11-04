use std::cmp::Ordering;

use crate::graph::Edge;
use crate::random::xorshift_rng as random;
use crate::graph::Digraph;
use crate::container::Heap;

#[derive(Clone)]
struct EdgeLink<'a> {
    edge: &'a Edge,
    previous_vertex_edge: usize,
    distance: f32,
}

impl<'a> PartialEq for EdgeLink<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.distance.eq(&other.distance)
    }
}

impl<'a> PartialOrd for EdgeLink<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

struct KShortestPathsIter<'a> {
    current: Option<&'a EdgeLink<'a>>,
    edge_to: &'a Vec<Vec<EdgeLink<'a>>>,
    source: usize,
    path_order: usize,
}

/// Provides edges in reverse order but without copying.
impl<'a> Iterator for KShortestPathsIter<'a> {
    type Item = &'a Edge;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take();
        self.current = current.filter(|x| x.edge.from > 0)
            .map(|x| &self.edge_to[x.edge.from][x.previous_vertex_edge]);
        current.map(|x| x.edge)
    }
}

pub struct KShortestPaths<'a> {
    source: usize,
    edge_to: Vec<Vec<EdgeLink<'a>>>, // mean to be sorted
}

impl<'a> KShortestPaths<'a> {
    fn new(graph: &'a Digraph, max_paths: usize, source: usize) -> Self {
        let mut edge_to = vec![Vec::new(); graph.len()];
        let mut heap = Heap::min();
        graph.adj(source).for_each(|x| heap.push(EdgeLink{ edge: x, previous_vertex_edge: 0, distance: x.weight }));
        while let Some(edge_link) = heap.pop() {
            let vertex = edge_link.edge.to;
            if edge_to[vertex].len() >= max_paths {
                continue;
            }

            edge_to[vertex].push(edge_link.clone());
            graph.adj(vertex).for_each(|next_edge| {
                let next_edge_link = EdgeLink {
                    edge: next_edge,
                    previous_vertex_edge: edge_to[vertex].len() - 1,
                    distance: edge_link.distance + next_edge.weight,
                };

                heap.push(next_edge_link);
            });
        }

        Self { edge_to, source }
    }

    fn path_to(&self, vertex: usize, path_order: usize) -> KShortestPathsIter {
        KShortestPathsIter {
            edge_to: &self.edge_to,
            current: self.edge_to.get(vertex).and_then(|x| x.get(path_order)),
            source: self.source,
            path_order,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn different_paths() {
        let edges = vec![
            Edge{from: 0, to: 1, weight: 0.1},
            Edge{from: 1, to: 2, weight: 0.1},
            Edge{from: 2, to: 4, weight: 0.1},

            Edge{from: 0, to: 3, weight: 0.2},
            Edge{from: 3, to: 4, weight: 0.2},
            Edge{from: 4, to: 5, weight: 0.1},
        ];

        let mut di = Digraph::new(6);
        edges.into_iter().for_each(|x| di.add(x));
        let k_shortest_path = KShortestPaths::new(&di, 2, 0);

        let mut path51 = k_shortest_path.path_to(5, 1);
        assert_eq!(path51.next(), Some(&Edge{from: 4, to: 5, weight: 0.1}));
        assert_eq!(path51.next(), Some(&Edge{from: 3, to: 4, weight: 0.2}));
        assert_eq!(path51.next(), Some(&Edge{from: 0, to: 3, weight: 0.2}));
        assert_eq!(path51.next(), None);

        let mut path50 = k_shortest_path.path_to(5, 0);
        assert_eq!(path50.next(), Some(&Edge{from: 4, to: 5, weight: 0.1}));
        assert_eq!(path50.next(), Some(&Edge{from: 2, to: 4, weight: 0.1}));
        assert_eq!(path50.next(), Some(&Edge{from: 1, to: 2, weight: 0.1}));
        assert_eq!(path50.next(), Some(&Edge{from: 0, to: 1, weight: 0.1}));
        assert_eq!(path50.next(), None);
    }

    #[test]
    fn same_nodes_different_paths() {
        let edges = vec![
            Edge{from: 0, to: 1, weight: 0.1},
            Edge{from: 0, to: 1, weight: 0.2},
            Edge{from: 0, to: 1, weight: 0.3},

            Edge{from: 1, to: 2, weight: 0.1},
            Edge{from: 1, to: 2, weight: 0.2},
            Edge{from: 1, to: 2, weight: 0.3},

            Edge{from: 2, to: 3, weight: 0.1},
            Edge{from: 2, to: 3, weight: 0.2},
            Edge{from: 2, to: 3, weight: 0.3},
        ];

        let mut di = Digraph::new(4);
        edges.into_iter().for_each(|x| di.add(x));
        let k_shortest_path = KShortestPaths::new(&di, 2, 0);
        assert_eq!(k_shortest_path.edge_to[3].len(), 2);
        assert_eq!(k_shortest_path.edge_to[3][0].distance, 0.3);
        assert_eq!(k_shortest_path.edge_to[3][1].distance, 0.4);

        assert_eq!(k_shortest_path.edge_to[2].len(), 2);
        assert_eq!(k_shortest_path.edge_to[2][0].distance, 0.2);
        assert_eq!(k_shortest_path.edge_to[2][1].distance, 0.3);

        assert_eq!(k_shortest_path.edge_to[1].len(), 2);
        assert_eq!(k_shortest_path.edge_to[1][0].distance, 0.1);
        assert_eq!(k_shortest_path.edge_to[1][1].distance, 0.2);

        assert_eq!(k_shortest_path.path_to(3, 3).next(), None);

        let mut path31 = k_shortest_path.path_to(3, 1);
        assert_eq!(path31.next(), Some(&Edge{from: 2, to: 3, weight: 0.2}));
        assert_eq!(path31.next(), Some(&Edge{from: 1, to: 2, weight: 0.1}));
        assert_eq!(path31.next(), Some(&Edge{from: 0, to: 1, weight: 0.1}));
        assert_eq!(path31.next(), None);
    }
}
