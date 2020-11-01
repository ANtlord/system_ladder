use std::f32::INFINITY;
use std::cmp::Ordering;

use crate::container::Heap;
use crate::graph::Digraph;
use crate::graph::Edge;
use crate::utils::quicksort;

#[derive(Clone)]
enum EdgeReference<'a> {
    Index(usize),
    Ptr(Box<EdgeLink<'a>>),
}

#[derive(Clone)]
struct EdgeLink<'a> {
    edge: &'a Edge,
    previous: Option<EdgeReference<'a>>,
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

type EdgeLinks<'a> = Vec<Option<EdgeLink<'a>>>;

pub struct Monotonic<'a> {
    edge_to: EdgeLinks<'a>,
}

struct EdgeLinkIter<'a>{
    cursor: Option<&'a EdgeLink<'a>>,
    edge_to: &'a EdgeLinks<'a>,
}

impl<'a> Iterator for EdgeLinkIter<'a> {
    type Item = &'a Edge;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.cursor.take();
        let previous = current.and_then(|x| x.previous.as_ref());
        self.cursor = previous.and_then(|prev| match prev {
            EdgeReference::Index(index) => self.edge_to[*index].as_ref(),
            EdgeReference::Ptr(ptr) => Some(ptr.as_ref()),
        });

        current.map(|x| x.edge)
    }
}

impl<'a> Monotonic<'a> {
    fn new(graph: &'a Digraph, ascending: bool) -> Self {
        let comp = if ascending { |x, y| x < y } else { |x, y| x > y };
        let mut heap = Heap::min();
        let mut visited_edges_counters = vec![0usize; graph.len()];
        let mut edge_to: EdgeLinks<'a> = vec![None; graph.len()];
        graph.adj(0).for_each(|x| heap.push(EdgeLink{ edge: x, previous: None, distance: x.weight }));
        while let Some(edge_link) = heap.pop() {
            let vertex = edge_link.edge.to;
            if edge_to[vertex].as_ref().map_or(INFINITY, |x| x.distance) > edge_link.distance {
                edge_to[vertex] = Some(edge_link.clone());
            }

            let mut next_edges = graph.adj(vertex).collect::<Vec<_>>();
            // sorting allows to avoid checking of distances of all adjacement edges of all
            // adjacement edges of the current edge.
            // It's n^2 to determine which of edges is eligable to relax. Example of searching
            // descending shortest paths
            //
            //       A -- 0.2 -> C -- 0.5 --> D
            //      /           / \
            //     /           /   \
            //    /           /    0.1
            //  0.3         0.6      \
            //  /           /         E
            // S -- 0.9 -> B
            //
            // Consider vertex C. A-C comes first and takes C-E as (C-E).weight < (A-C).weight. Then B-C
            // comes into. It's able to take both of the edges as weight of them is lower than
            // (B-C).weight but C-E is took already so B-C must take only C-D.
            //
            // Without sorting it requires checking distances to all adjacement of all
            // adjacement edges of vertex B. If we found them we relax B-C. When S-B
            // comes into it checks distance of C which is 0.5 so it's lower than 1.5 then
            // no reason to consider B-C but algorithm misses C-D. To fix the mistake it requires
            // to check C-D and C-E then if distance to one of them is greater than distance to C +
            // edge weight then we relax B-C.
            //
            // Problem is having not only B-C but having B-C1, B-C2 ... B-Cn in this case it
            // requires to check all edges which go from Ci which means n^2 running time instead of nlog(n)
            // consider sorting.
            //
            // TODO: POSSIBLE OPTIMIZATION from nlog(n) to n:
            // Do not sort edges and check them all consider weight.
            // Every checked edge can be moved in the in the end of the bag of the adjacement
            // edges and counted. Then it iterates edges while number of iteration is less than
            // difference of number of edges and the counter. As we have counters already for every
            // vertex (visited_edges_counters) it doesn't require additional memory. Possible
            // problem is mutation of digraph but it gets initial state after the process.
            //
            // Requires:
            // - add method `adj_mut` which allows to get an edge of the vertex and move it to the
            // tail of its list after use optionaly. (Probably need a some sort of container with a
            // flag indicates need of moving to the tail)
            // - add method `move_to_end` or `swap`. `move_to_end` requires doubly linked list to be
            // used under the hood of the Bag but `swap` requires dynamic array (aka Vec).

            // descending path - sort ascending and vice versa
            quicksort(&mut next_edges, |x, y| comp(y.weight, x.weight));
            let processed_edges_count = visited_edges_counters[vertex];
            next_edges[processed_edges_count .. ].iter().take_while(|x| comp(edge_link.edge.weight, x.weight)).for_each(|next_edge| {
                visited_edges_counters[vertex] += 1;
                let next_vertex_distance = edge_link.distance + next_edge.weight;
                let previous = Some(if edge_to[vertex].as_ref().map(|x| x.edge == edge_link.edge).unwrap_or(false) {
                    EdgeReference::Index(vertex)
                } else {
                    EdgeReference::Ptr(Box::new(edge_link.clone()))
                });

                let next_edge_link = EdgeLink{ edge: next_edge, previous, distance: next_vertex_distance };
                heap.push(next_edge_link);
            });
        }

        Self { edge_to }
    }

    fn path_to(&self, vertex: usize) -> EdgeLinkIter {
        EdgeLinkIter{
            cursor: self.edge_to[vertex].as_ref(),
            edge_to: &self.edge_to,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;


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
                    let actual = shortest_path.edge_to[self.target].as_ref().map(|x| x.edge);
                    assert_eq!(actual, Some(expected));
                },
                None => assert!(shortest_path.edge_to[self.target].is_none()),
            }

            let actual_distance = shortest_path.edge_to[self.target].as_ref().map(|x| x.distance);
            assert!(actual_distance.is_some());
            assert_eq!(actual_distance.unwrap(), self.expected_distance);
        }
    }

    #[test]
    fn shortest_wrong_ascending() {
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
    fn shortest_right_ascending() {
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
    fn shortest_right_descending() {
        const TARGET: usize = 4;
        let edges = vec![
            Edge{from: 0, to: 1, weight: 0.6},
            Edge{from: 1, to: 2, weight: 0.3},

            Edge{from: 0, to: 2, weight: 0.3},
            Edge{from: 2, to: 3, weight: 0.2},
            Edge{from: 3, to: TARGET, weight: 0.1},
        ];
        Test{
            target: TARGET,
            expected_edge: Some(edges[4].clone()),
            expected_distance: edges[2..].iter().map(|x| x.weight).sum(),
            edges,
            ascending: false,
        }.run();
    }

    #[test]
    fn alternative_right_descending() {
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

    fn dist(el: &EdgeLink) -> f32 {
        el.distance
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
        assert_eq!(shortest_path.edge_to[5].as_ref().map(dist), Some(fifth_vertex_distance));
        assert_eq!(shortest_path.edge_to[4].as_ref().map(dist), Some(third_vertex_distance));
        assert_eq!(shortest_path.edge_to[3].as_ref().map(dist), Some(fourth_vertex_distance));

        let mut iter = shortest_path.path_to(5);
        assert_eq!(iter.next(), Some(&Edge{from: 1, to: 5, weight: 0.4}));
        assert_eq!(iter.next(), Some(&Edge{from: 2, to: 1, weight: 0.6}));
        assert_eq!(iter.next(), Some(&Edge{from: 0, to: 2, weight: 0.7}));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn three_descending_paths_through_one_vertex() {
        // visualize dot ./assets/memory.dot -Tsvg > memory.svg
        let edges = vec![
            Edge{from: 0, to: 1, weight: 0.2},
            Edge{from: 1, to: 2, weight: 0.1},

            Edge{from: 0, to: 1, weight: 0.3},
            Edge{from: 1, to: 3, weight: 0.2},

            Edge{from: 0, to: 1, weight: 0.4},
            Edge{from: 1, to: 4, weight: 0.3},
        ];

        let mut graph = Digraph::new(5);
        edges.into_iter().for_each(|x| graph.add(x));
        let shortest_path = Monotonic::new(&graph, false);
        let mut path4 = shortest_path.path_to(4);
        assert_eq!(path4.next(), Some(&Edge{from: 1, to: 4, weight: 0.3}));
        assert_eq!(path4.next(), Some(&Edge{from: 0, to: 1, weight: 0.4}));
        assert_eq!(path4.next(), None);

        let mut path3 = shortest_path.path_to(3);
        assert_eq!(path3.next(), Some(&Edge{from: 1, to: 3, weight: 0.2}));
        assert_eq!(path3.next(), Some(&Edge{from: 0, to: 1, weight: 0.3}));
        assert_eq!(path3.next(), None);

        let mut path2 = shortest_path.path_to(2);
        assert_eq!(path2.next(), Some(&Edge{from: 1, to: 2, weight: 0.1}));
        assert_eq!(path2.next(), Some(&Edge{from: 0, to: 1, weight: 0.2}));
        assert_eq!(path2.next(), None);
    }

    #[test]
    fn memory() {
        // visualize dot ./assets/memory.dot -Tsvg > memory.svg
        let edges = vec![
            Edge{from: 0, to: 1, weight: 0.2},
            Edge{from: 1, to: 2, weight: 0.1},

            Edge{from: 0, to: 1, weight: 0.3},
            Edge{from: 1, to: 3, weight: 0.2},

            Edge{from: 0, to: 1, weight: 0.4},
            Edge{from: 1, to: 4, weight: 0.3},
        ];

        let edge_reference_index = EdgeReference::Index(1);
        assert_eq!(mem::size_of_val(&edge_reference_index), 16);
        let edge_link = EdgeLink{ edge: &edges[0], previous: None, distance: 0f32 };
        assert_eq!(mem::size_of_val(&edge_link), 32);

        let edge_link_previous = EdgeLink{ edge: &edges[0], previous: Some(edge_reference_index), distance: 0f32 };
        assert_eq!(mem::size_of_val(&edge_link_previous), 32);

        let edge_reference_ptr = EdgeReference::Ptr(Box::new(edge_link_previous));
        assert_eq!(mem::size_of_val(&edge_reference_ptr), 16);

        let mut graph = Digraph::new(5);
        edges.into_iter().for_each(|x| graph.add(x));
        let shortest_path = Monotonic::new(&graph, false);
        // without alternative edges which allocated on the heap. Actual size is 32 * 5 bytes
        assert_eq!(mem::size_of_val(&shortest_path), 24);
    }
}
