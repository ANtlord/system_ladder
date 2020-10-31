use std::f32::INFINITY;
use std::cmp::Ordering;

use crate::container::Heap;
use crate::graph::Digraph;
use crate::graph::Edge;
use crate::utils::quicksort;

#[derive(Copy, Clone)]
struct EdgeLink<'a> {
    edge: &'a Edge,
    previous: Option<&'a Edge>,
}

type EdgeLinks<'a> = Vec<Option<EdgeLink<'a>>>;

pub struct Monotonic<'a> {
    edge_to: EdgeLinks<'a>,
    dist_to: Vec<f32>,
}

struct QueuedEdge<'a>(&'a Edge, f32);

impl<'a> PartialEq for QueuedEdge<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.1.eq(&other.1)
    }
}

impl<'a> PartialOrd for QueuedEdge<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.1.partial_cmp(&other.1)
    }
}

impl<'a> Monotonic<'a> {
    fn new(graph: &'a Digraph, ascending: bool) -> Self {
        let comp = if ascending { |x, y| x < y } else { |x, y| x > y };
        let mut heap = Heap::min();
        let mut visited_edges_counters = vec![0usize; graph.len()];
        let mut edge_to: EdgeLinks<'a> = vec![None; graph.len()];
        let mut dist_to = vec![INFINITY; graph.len()];
        dist_to[0] = 0.;
        graph.adj(0).for_each(|x| heap.push(QueuedEdge(x, x.weight)));

        while let Some(QueuedEdge(edge, distance)) = heap.pop() {
            let vertex = edge.to; // 2
            if dist_to[vertex] > distance { // inf > 0.1
                dist_to[vertex] = distance;
            }

            // TODO: Running time checking every adjacement edge can be optimized for free in term of memory.
            // Every checked edge can be moved in the in the end of the bag of the adjacement
            // edges and counted. Then it iterates edges while number of iteration is less than
            // difference of number of edges and the counter. As we have counters already for every
            // vertex (visited_edges_counters) it doesn't require additional memory.
            //
            // Requires:
            // - add method `adj_mut` which allows to get an edge of the vertex and move it to the
            // tail of its list after use optionaly. (Probably need a some sort of container with a
            // flag indicates need of moving to the tail)
            // - add method `move_to_end` or `swap`. `move_to_end` requires doubly linked list to be
            // used under the hood of the Bag but `swap` requires dynamic array (aka Vec).

            // [&Edge{from: 2, to: 3, weight: 0.2}]
            let mut next_edges = graph.adj(vertex).collect::<Vec<_>>();
            // descending - sort ascending and vice versa
            quicksort(&mut next_edges, |x, y| comp(y.weight, x.weight));
            let processed_edges_count = visited_edges_counters[vertex];
            next_edges[processed_edges_count .. ].iter().filter(|x| comp(edge.weight, x.weight)).for_each(|next_edge| {
                let next_vertex_distance = distance + next_edge.weight;
                visited_edges_counters[vertex] += 1;
                if dist_to[next_edge.to] > next_vertex_distance { 
                    // dist_to[next_edge.to] = next_vertex_distance;
                    edge_to[next_edge.to] = Some(EdgeLink{ edge: next_edge, previous: Some(edge) });
                }

                heap.push(QueuedEdge(next_edge, next_vertex_distance));

            });
        }

        Self { edge_to, dist_to }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Test {
        target: usize,
        expected_edge: Option<Edge>,
        expected_distance: f32,
        edges: Vec<Edge>,
        ascending: bool,
    }

    impl Test {
        fn run(self) {
            let mut graph = Digraph::new(self.edges.len());
            self.edges.into_iter().for_each(|x| graph.add(x));
            let shortest_path = Monotonic::new(&graph, self.ascending);
            match self.expected_edge {
                Some(ref expected) => {
                    let actual = shortest_path.edge_to[self.target].map(|x| x.edge);
                    assert_eq!(actual, Some(expected));
                },
                None => assert!(shortest_path.edge_to[self.target].is_none()),
            }

            assert_eq!(shortest_path.dist_to[self.target], self.expected_distance);
        }
    }

    #[test]
    fn shortest_wrong() {
        const TARGET: usize = 5;
        let edges = vec![
            Edge{from: 0, to: 1, weight: 0.3},
            Edge{from: 1, to: 2, weight: 0.4},
            Edge{from: 2, to: TARGET, weight: 0.5},

            Edge{from: 0, to: 3, weight: 0.1},
            Edge{from: 3, to: 4, weight: 0.2},
            Edge{from: 4, to: TARGET, weight: 0.1},
        ];
        Test{
            target: TARGET,
            expected_edge: Some(edges[2].clone()),
            expected_distance: edges[..3].iter().map(|x| x.weight).sum(),
            edges,
            ascending: true,
        }.run();
    }

    #[test]
    fn shortest_right() {
        const TARGET: usize = 5;
        let edges = vec![
            Edge{from: 0, to: 1, weight: 0.3},
            Edge{from: 1, to: 2, weight: 0.4},
            Edge{from: 2, to: TARGET, weight: 0.5},

            Edge{from: 0, to: 3, weight: 0.1},
            Edge{from: 3, to: 4, weight: 0.2},
            Edge{from: 4, to: TARGET, weight: 0.3},
        ];
        Test{
            target: TARGET,
            expected_edge: Some(edges[5].clone()),
            expected_distance: edges[3..].iter().map(|x| x.weight).sum(),
            edges,
            ascending: true,
        }.run();
    }

    #[test]
    fn tricky() {
        const TARGET: usize = 4;
        let edges = vec![
            Edge{from: 0, to: 1, weight: 0.6},
            Edge{from: 1, to: 2, weight: 0.3},

            Edge{from: 0, to: 2, weight: 0.1},
            Edge{from: 2, to: 3, weight: 0.2},
            Edge{from: 3, to: TARGET, weight: 0.1},
        ];

        Test{
            target: TARGET,
            expected_edge: Some(edges[4].clone()),
            expected_distance: edges[..2].iter().chain(edges[3..].iter()).map(|x| x.weight).sum(),
            edges,
            ascending: false,
        }.run();
    }

    #[test]
    fn descending_from_two_paths() {
        let edges = vec![
            Edge{from: 0, to: 1, weight: 0.2},
            Edge{from: 0, to: 2, weight: 0.7},
            Edge{from: 2, to: 1, weight: 0.6},

            Edge{from: 1, to: 3, weight: 0.5},
            Edge{from: 1, to: 4, weight: 0.1},
            Edge{from: 1, to: 5, weight: 0.4},
        ];

        let fifth_vertex_distance = [edges[1].weight, edges[2].weight, edges[5].weight].iter().sum();
        let fourth_vertex_distance = [edges[1].weight, edges[2].weight, edges[3].weight].iter().sum();
        let third_vertex_distance = [edges[0].weight, edges[4].weight].iter().sum();
        let mut graph = Digraph::new(edges.len());
        edges.into_iter().for_each(|x| graph.add(x));
        let shortest_path = Monotonic::new(&graph, false);
        assert_eq!(shortest_path.dist_to[5], fifth_vertex_distance);
        assert_eq!(shortest_path.dist_to[4], third_vertex_distance);
        assert_eq!(shortest_path.dist_to[3], fourth_vertex_distance);
    }
}
