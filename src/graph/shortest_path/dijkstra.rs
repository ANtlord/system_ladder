use crate::graph::Digraph;
use crate::graph::Edge;
use crate::container::IndexedHeap;
use std::f32::INFINITY;

pub struct EdgeIter<'a> {
    edge_to: &'a Vec<Option<&'a Edge>>,
    current: Option<&'a Edge>,
}

impl<'a> Iterator for EdgeIter<'a> {
    type Item = &'a Edge;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take();
        self.current = current.and_then(|x| self.edge_to[x.from]);
        current
    }
}

pub struct Dijkstra<'a>{
    edge_to: Vec<Option<&'a Edge>>,
    dist_to: Vec<f32>,
}

impl<'a> Dijkstra<'a> {
    fn new(graph: &'a Digraph, source: usize) -> Result<Self, String> {
        let mut edge_to = vec![None; graph.len()];
        let mut dist_to = vec![INFINITY; graph.len()];
        dist_to[source] = 0.;
        let mut heap = IndexedHeap::new(Box::new(|x, y| x > y));
        heap.insert(dist_to[source], source);

        while let Some((distance, vertex)) = heap.pop() {
            // dist_to[vertex] = distance;
            for edge in graph.adj(vertex) {
                if dist_to[edge.to] > dist_to[vertex] + edge.weight {
                    dist_to[edge.to] = dist_to[vertex] + edge.weight;
                    // dbg!(source, edge.to);
                    edge_to[edge.to] = Some(edge);
                    // TODO: could be replaced by the checking before the loop.
                    // if dist_to[vertex] < distance then skip current iteration.
                    // in this case no need of indexed heap.
                    heap.change_key(&edge.to, dist_to[edge.to])
                        .unwrap_or_else(|_| heap.insert(dist_to[edge.to], edge.to));
                }
            }
        }

        // let len = edge_to.len();
        // let edge_to: Vec<&'a Edge> = edge_to.into_iter()
        //     .filter(Option::is_some).map(Option::unwrap).collect();
        // if len - 1 > edge_to.len() {
        //     return Err(format!("No edge to {} vertex", edge_to.len()));
        // }

        Ok(Self { edge_to, dist_to })
    }
    
    fn path_to(&self, target: usize) -> EdgeIter {
        EdgeIter{
            edge_to: &self.edge_to,
            current: self.edge_to.get(target).and_then(|x| *x),
        }
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

        let shortest_path = Dijkstra::new(&di, 0);
        assert!(shortest_path.is_ok(), shortest_path.err().unwrap());
        let shortest_path = shortest_path.unwrap();
        let expected_dist_to = [0.0, 0.4, 0.1];
        let expected_edge_to_from = [None, Some(2), Some(0)];
        assert_eq!(shortest_path.dist_to, expected_dist_to);
        assert_eq!(shortest_path.edge_to.iter().map(|x| x.map(|y| y.from)).collect::<Vec<_>>(), expected_edge_to_from);
    }

    #[test]
    fn non_zero_source() {
        let mut di = Digraph::new(4);
        di.add(Edge{from: 1, to: 0, weight: 0.1});
        di.add(Edge{from: 2, to: 1, weight: 0.2});
        di.add(Edge{from: 3, to: 2, weight: 0.1});
        di.add(Edge{from: 3, to: 0, weight: 0.4});
        let shortest_path = Dijkstra::new(&di, 3);
        assert!(shortest_path.is_ok(), shortest_path.err().unwrap());
    }

    mod skip_edge {
        use super::*;
        use std::mem;

        struct SkipEdge {
            sp: Dijkstra<'static>,
            di: Box<Digraph>,
            source: usize,
        }

        unsafe fn extend_lifetime<'a, T>(a: &'a T) -> &'static T {
            mem::transmute::<&'a T, &'static T>(a)
        }

        use crate::tprintln;

        impl SkipEdge {
            // the best to borrow edges to digraph instead. This helps to save unnessesary copies.
            fn new(edges: Vec<Edge>, source: usize) -> Result<Self, String> {
                let mut di = Box::new(Digraph::new(edges.len()));
                edges.iter().for_each(|x| di.add(x.clone()));
                let di_static: &'static Digraph = unsafe {
                    extend_lifetime(di.as_ref())
                };

                let sp = Dijkstra::new(&di_static, 0)?;
                Ok(Self { di, sp, source })
            }

            fn skip(&self, from: usize, to: usize, target: usize) -> Result<Vec<Edge>, String> {
                let mut di = Digraph::new(self.di.len());
                let edge_iters = (0 .. self.di.len()).filter(|x| x != &self.source).map(|x| self.di.adj(x));
                edge_iters.flatten().for_each(|x| di.add(Edge{from: x.to, to: x.from, weight: x.weight}));
                let reverse_sp = Dijkstra::new(&di, target)?;
                let alternative_dist = self.sp.dist_to[from] + reverse_sp.dist_to[to];
                let dist = self.sp.dist_to[target];

                if alternative_dist < dist {
                    let mut edges_from = self.sp.path_to(from).map(|x| x.clone()).collect::<Vec<_>>();
                    edges_from.reverse();
                    let mut edges_to = reverse_sp.path_to(to).map(|x| x.clone()).collect::<Vec<_>>();
                    edges_from.push(Edge{from, to, weight: 0.});
                    edges_from.append(&mut edges_to);
                    return Ok(edges_from);
                }

                let mut edges = self.sp.path_to(target).map(|x| x.clone()).collect::<Vec<_>>();
                edges.reverse();
                Ok(edges)
            }
        }

        #[test]
        fn basic() {
            let edges = vec![
                Edge{from: 0, to: 1, weight: 0.1},
                Edge{from: 1, to: 2, weight: 0.2},
                Edge{from: 2, to: 3, weight: 0.1},
                Edge{from: 0, to: 3, weight: 0.4},
            ];

            let program = SkipEdge::new(edges, 0);
            assert!(!program.is_err(), program.err().unwrap());
            let res = program.unwrap().skip(0, 3, 3);
            assert_eq!(res, Ok(vec![
                Edge{from: 0, to: 3, weight: 0.}
            ]));
        }
    }
}
