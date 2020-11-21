use std::collections::VecDeque;
use std::f64::INFINITY;

use crate::graph::FlowNetwork;
use crate::graph::FlowEdge;

type Edges<'a> = Vec<Option<&'a FlowEdge>>;

struct FordFulkerson<'a> {
    edge_to: Edges<'a>,
    maxflow: f64, // this is mincut as well.
    marked: Vec<bool>, // every true stays with `from` (A cut), every false stays with `to` (B cut)
}

fn adding_flow_err(e: &str, current: usize) -> String {
    format!("adding flow: fail getting other point of edge of vertex {}: {}", current, e)
}

fn other_err(e: &str, current: usize) -> String {
    format!("get bottleneck: fail getting other point of edge of vertex {}: {}", current, e)
}

fn no_edge(edge_index: usize, vertex: usize) -> String {
    format!("fail other edge {} adjacted to vertex {}", edge_index, vertex)
}

fn failed_path(e: impl AsRef<str>) -> String {
    format!("fail getting augumenting path: {}", e.as_ref())
}

fn get_bottleneck(edge_to: &[Option<usize>], net: &FlowNetwork, from: usize, to: usize) -> Result<f64, String> {
    let mut current = to;
    let mut bottleneck = INFINITY;
    while let Some(edge_index) = edge_to[current] {
        let edge = net.edge(edge_index).ok_or(format!("fail getting edge {}", edge_index))?;
        let capacity = edge.residual_capacity_to(current)
            .map_err(|e| format!("fail getting residual_capacity_to {}: {}", current, e))?;
        bottleneck = bottleneck.min(capacity);
        current = edge.other(current).map_err(|e| other_err(&e, current))?;
    }
    
    Ok(bottleneck)
}

impl<'a> FordFulkerson<'a> {
    fn new(net: &'a mut FlowNetwork, from: usize, to: usize) -> Result<Self, String> {
        if from >= net.len() || to >= net.len() {
            Err(format!("invalid from = {} or to = {} as net.len = {}", from, to, net.len()))?;
        }

        let mut maxflow = 0.;
        let (edge_to, marked) = loop {
            let (edge_to, marked) = has_augumenting_path(net, from, to).map_err(failed_path)?;
            if !marked[to] {
                break (edge_to, marked)
            }

            let bottleneck = get_bottleneck(&edge_to, net, from, to)?;
            let mut current = to;
            while let Some(edge_index) = edge_to[current] {
                let mut edge = net.edge_mut(edge_index).ok_or(format!("fail getting mutable edge {}", edge_index))?;
                edge.add_residual_flow_to(current, bottleneck)
                    .map_err(|e| format!("fail add_residual_flow_to to {}: {}", current, e))?;
                current = edge.other(current).map_err(|e| adding_flow_err(&e, current))?;
            }

            maxflow += bottleneck;
        };

        let netref: &FlowNetwork = net;
        Ok(Self { marked, edge_to: edge_to.into_iter().map(|x| x.and_then(|i| netref.edge(i))).collect(), maxflow })
    }

}

fn has_augumenting_path(net: &FlowNetwork, from: usize, to: usize) -> Result<(Vec<Option<usize>>, Vec<bool>), String> {
    let mut edge_to = vec![None; net.len()];
    let mut marked = vec![false; net.len()];
    marked[from] = true;
    let mut queue = VecDeque::new();
    queue.push_back(from);
    while let Some(vertex) = queue.pop_front() {
        for edge_index in net.adj(vertex) {
            let edge = net.edge(*edge_index).ok_or(no_edge(*edge_index, vertex))?;
            let other = edge.other(vertex).map_err(|e| format!("fail getting other vertex: {}", e))?;
            let cap = edge.residual_capacity_to(other).map_err(|e| format!("fail getting capacity: {}", e))?;
            if cap > 0. && !marked[other] {
                edge_to[other] = Some(*edge_index);
                marked[other] = true;
                queue.push_back(other);
            }
        }
    }

    Ok((edge_to, marked))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    fn assert_res<T, E: Debug>(res: Result<T, E>) -> T {
        match res {
            Err(ref e) => assert!(false, "{:?}", e),
            _ => (),
        }
        
        res.unwrap()
    }

    #[test]
    fn line() {
        let source = 0;
        let target = 2;
        let edges = vec![
            FlowEdge::new(source, 1, 5.),
            FlowEdge::new(1, target, 10.),
        ];

        let mut net = FlowNetwork::new(target + 1);
        edges.iter().for_each(|x| net.add(x.clone()));
        assert_eq!(net.adj(source).collect::<Vec<_>>(), vec!(&0));
        let ff = assert_res(FordFulkerson::new(&mut net, source, target));
        assert_eq!(ff.maxflow, 5.);
        assert_eq!(ff.marked, vec![true, false, false]);
    }

    fn get_edge_flow(from_vertex: usize, to_vertex: usize, net: &FlowNetwork) -> f64 {
        let edges: Vec<_> = net.adj(from_vertex).filter_map(|x| net.edge(*x))
            .filter(|edge| edge.other(from_vertex) == Ok(to_vertex)).collect();
        assert_eq!(edges.len(), 1);
        edges[0].residual_capacity_to(from_vertex).unwrap()
    }

    fn test_edge_flow(from_vertex: usize, to_vertex: usize, net: &FlowNetwork, expected_flow: f64) {
        assert_eq!(get_edge_flow(from_vertex, to_vertex, net), expected_flow);
    }

    #[test]
    fn basic() {

        // see assets/maxflow/basic.dot
        let source = 0;
        let target = 7;
        let edges = vec![
            FlowEdge::new(source, 1, 10.),
            FlowEdge::new(source, 2, 5.),
            FlowEdge::new(source, 3, 15.),

            FlowEdge::new(1, 2, 4.),
            FlowEdge::new(2, 3, 4.),

            FlowEdge::new(1, 5, 15.),

            FlowEdge::new(1, 4, 9.),
            FlowEdge::new(2, 5, 8.),
            FlowEdge::new(3, 6, 16.),

            FlowEdge::new(6, 2, 6.),

            FlowEdge::new(4, 5, 15.),
            FlowEdge::new(5, 6, 15.),

            FlowEdge::new(4, target, 10.),
            FlowEdge::new(5, target, 10.),
            FlowEdge::new(6, target, 10.),
        ];

        let mut net = FlowNetwork::new(8);
        edges.into_iter().for_each(|x| net.add(x));
        let ff = assert_res(FordFulkerson::new(&mut net, source, target));
        let FordFulkerson{ edge_to, maxflow, marked } = ff;

        assert!(marked[0]);
        assert!(marked[2]);
        assert!(marked[3]);
        assert!(marked[6]);
        
        test_edge_flow(source, 1, &net, 10.); // mincut edge
        test_edge_flow(source, 2, &net, 5.);
        test_edge_flow(source, 3, &net, 13.);

        test_edge_flow(1, 2, &net, 0.);
        test_edge_flow(2, 3, &net, 0.);

        test_edge_flow(1, 5, &net, 2.);

        test_edge_flow(1, 4, &net, 8.);
        test_edge_flow(2, 5, &net, 8.); // mincut edge
        test_edge_flow(3, 6, &net, 13.);

        test_edge_flow(6, 2, &net, 3.);

        test_edge_flow(4, 5, &net, 0.);
        test_edge_flow(5, 6, &net, 0.);

        test_edge_flow(4, target, &net, 8.);
        test_edge_flow(5, target, &net, 10.);
        test_edge_flow(6, target, &net, 10.); // mincut edge

        assert!(!marked[1]);
        assert!(!marked[4]);
        assert!(!marked[5]);
        assert!(!marked[7]);

        assert_eq!(maxflow, 28.);
        let target_input_flow = get_edge_flow(4, target, &net) + get_edge_flow(5, target, &net) + get_edge_flow(6, target, &net);
        assert_eq!(maxflow, target_input_flow);
        let source_output_flow = get_edge_flow(source, 1, &net) + get_edge_flow(source, 2, &net) + get_edge_flow(source, 3, &net);
        assert_eq!(maxflow, source_output_flow);
        let mincut_flow = get_edge_flow(source, 1, &net) + get_edge_flow(2, 5, &net) + get_edge_flow(6, target, &net);
        assert_eq!(maxflow, mincut_flow);
    }

    mod bipartial_dance {
        use super::*;
        use crate::random::shuffle;

        struct Data {
            source: usize,
            target: usize,
            flow_network: FlowNetwork,
        }

        fn build_data(pair_count: usize, mutual_relationships_count: usize) -> Data {
            assert!(pair_count >= mutual_relationships_count,
                "number of relationships can't be greater number of people, {}, {}", pair_count, mutual_relationships_count);
            let mut people = Vec::with_capacity(pair_count + pair_count);
            (0 .. pair_count + pair_count).for_each(|x| people.push(x));
            let (source, target) = (people.len(), people.len() + 1);
            let network_size = people.len() + 2;
            let mut flow_network = FlowNetwork::new(network_size);
            shuffle(&mut people[.. pair_count]); // shuffle men
            shuffle(&mut people[pair_count ..]); // shuffle women
            let (men, women) = people.split_at(pair_count);
            women.iter().for_each(|x| flow_network.add(FlowEdge::new(*x, target, 1.)));
            men.iter().enumerate().for_each(|(i, man)| {
                flow_network.add(FlowEdge::new(source, *man, 1.));
                (i .. i + mutual_relationships_count).for_each(|j| {
                    flow_network.add(FlowEdge::new(*man, women[j % pair_count], 1.))
                });
            });

            Data { source, target, flow_network }
        }

        #[test]
        fn basic() {
            // sample for 2 mutual_relationships_count = 2 and pair_count = 5 as
            // assets/maxflow/bipartial_dance/basic.dot
            for mutual_relationships_count in 1 .. 10 {
                for pair_count in mutual_relationships_count .. 10 {
                    let mut data = build_data(pair_count, mutual_relationships_count);
                    assert_eq!(data.source, pair_count * 2);
                    assert_eq!(data.target, pair_count * 2 + 1);
                    let ff = assert_res(FordFulkerson::new(&mut data.flow_network, data.source, data.target));
                    let FordFulkerson{marked, maxflow, ..} = ff;
                    assert!(marked[data.source]);
                    let are_all_people_have_pair = !marked[.. data.source].iter().all(|x| *x);
                    assert!(are_all_people_have_pair);
                    assert_eq!(data.flow_network.adj(data.source).count(), pair_count);
                    assert_eq!(data.flow_network.adj(data.target).count(), pair_count);
                    assert_eq!(maxflow, pair_count as f64);
                }
            }
        }

    }

    mod find_the_biggest_closure {
        use super::*;

        struct Edge {
            from: usize,
            to: usize,
        }

        fn build_flow_network(
            source: usize,
            target: usize,
            edges: impl IntoIterator<Item=Edge>,
            weights: &[f64],
            size: usize,
        ) -> FlowNetwork {
            let mut flow_network = FlowNetwork::new(size);
            edges.into_iter().for_each(|x| flow_network.add(FlowEdge::new(x.from, x.to, INFINITY)));
            for vertex_index in 0 .. weights.len() {
                let weight = weights[vertex_index];
                if weight < 0. {
                    flow_network.add(FlowEdge::new(vertex_index, target, weight.abs()));
                } else {
                    flow_network.add(FlowEdge::new(source, vertex_index, weight));
                }
            }

            flow_network
        }

        #[test]
        fn rombus() {
            // assets/maxflow/find_the_biggest_closure/rombus.dot
            let (source, target) = (4, 5);
            let weights = vec![0.1, -10., 0.2, 0.3];
            let edges = vec![
                Edge { from: 0, to: 1 },
                Edge { from: 0, to: 2 },
                Edge { from: 1, to: 3 },
                Edge { from: 2, to: 3 },
            ];

            let mut net = build_flow_network(source, target, edges, &weights, 6);
            let ff = FordFulkerson::new(&mut net, source, target).unwrap();
            assert_eq!(ff.marked, vec![false, false, true, true, true, false]);
            assert_eq!(ff.maxflow, 0.1);
            let expected_biggest_closure_total_weight = weights[2] + weights[3];
            let biggest_closure_total_weight: f64 = weights.iter().enumerate()
                .filter(|(i, x)| ff.marked[*i]).map(|(_, x)| x).sum();
            assert_eq!(biggest_closure_total_weight, expected_biggest_closure_total_weight);
        }

        #[test]
        fn all_positive() { }

        #[test]
        fn all_negative() { }

        #[test]
        fn crystal() { }
    }
}
