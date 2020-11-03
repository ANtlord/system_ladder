use crate::graph::Digraph;
use crate::graph::Edge;
use crate::container::IndexedHeap;
use std::f32::INFINITY;

pub struct Dijkstra<'a>{
    edge_to: Vec<&'a Edge>,
    dist_to: Vec<f32>,
}

impl<'a> Dijkstra<'a> {
    fn new(graph: &'a Digraph) -> Result<Self, String> {
        let mut edge_to = vec![None; graph.len()];
        let mut dist_to = vec![INFINITY; graph.len()];
        dist_to[0] = 0.;
        let mut heap = IndexedHeap::new(Box::new(|x, y| x > y));
        heap.insert(dist_to[0], 0);

        while let Some((distance, vertex)) = heap.pop() {
            // dist_to[vertex] = distance;
            for edge in graph.adj(vertex) {
                if dist_to[edge.to] > dist_to[vertex] + edge.weight {
                    dist_to[edge.to] = dist_to[vertex] + edge.weight;
                    edge_to[edge.to] = Some(edge);
                    // TODO: could be replaced by the checking before the loop.
                    // if dist_to[vertex] < distance then skip current iteration.
                    // in this case no need of indexed heap.
                    heap.change_key(&edge.to, dist_to[edge.to])
                        .unwrap_or_else(|_| heap.insert(dist_to[edge.to], edge.to));
                }
            }
        }

        let len = edge_to.len();
        let edge_to: Vec<&'a Edge> = edge_to.into_iter().skip(1)
            .take_while(Option::is_some).map(Option::unwrap).collect();
        if len - 1 > edge_to.len() {
            return Err(format!("No edge to {} vertex", edge_to.len()));
        }

        Ok(Self { edge_to, dist_to })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut di = Digraph::new(3);
        di.add(Edge{from: 0, to: 1, weight: 0.9});
        di.add(Edge{from: 0, to: 2, weight: 0.1});
        di.add(Edge{from: 2, to: 1, weight: 0.3});

        let shortest_path = Dijkstra::new(&di);
        assert!(shortest_path.is_ok(), shortest_path.err().unwrap());
        let shortest_path = shortest_path.unwrap();
        let expected_dist_to = [0.0, 0.4, 0.1];
        let expected_edge_to_from = [2, 0];
        assert_eq!(shortest_path.dist_to, expected_dist_to);
        assert_eq!(shortest_path.edge_to.iter().map(|x| x.from).collect::<Vec<usize>>(), expected_edge_to_from);
    }
}
