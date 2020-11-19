use std::collections::VecDeque;
use std::f64::INFINITY;

use crate::graph::FlowNetwork;
use crate::graph::FlowEdge;

type Edges<'a> = Vec<Option<&'a FlowEdge>>;

struct FordFulkerson<'a> {
    edge_to: Edges<'a>,
    maxflow: f64,
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

    fn test_edge_flow(from_vertex: usize, to_vertex: usize, net: &FlowNetwork, expected_flow: f64) {
        let edges: Vec<_> = net.adj(from_vertex).filter_map(|x| net.edge(*x))
            .filter(|edge| edge.other(from_vertex) == Ok(to_vertex)).collect();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].residual_capacity_to(from_vertex), Ok(expected_flow));
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
        // mincut edges
        test_edge_flow(0, 1, &net, 10.);
        test_edge_flow(2, 5, &net, 8.);
        test_edge_flow(6, target, &net, 10.);

        assert!(!marked[1]);
        assert!(!marked[4]);
        assert!(!marked[5]);
        assert!(!marked[7]);
        assert_eq!(maxflow, 25.);
    }

    mod bipartial_dance {
        use super::*;
        use crate::random::shuffle;

        struct Data {
            source: usize,
            target: usize,
            flow_network: FlowNetwork,
        }

        fn build_data(men_women_count: usize, mutual_relationships_count: usize) -> Data {
            assert!(men_women_count >= mutual_relationships_count,
                "number of relationships can't be greated number of people, {}, {}", men_women_count, mutual_relationships_count);
            let mut people = Vec::with_capacity(men_women_count + men_women_count);
            (0 .. men_women_count + men_women_count).for_each(|x| people.push(x));
            let (source, target) = (people.len(), people.len() + 1);
            let network_size = people.len() + 2;
            let mut flow_network = FlowNetwork::new(network_size);
            shuffle(&mut people[.. men_women_count]); // shuffle men
            shuffle(&mut people[men_women_count ..]); // shuffle women
            let (men, women) = people.split_at(men_women_count);
            women.iter().for_each(|x| flow_network.add(FlowEdge::new(*x, target, 1.)));
            men.iter().for_each(|x: &usize| {
                flow_network.add(FlowEdge::new(source, *x, 1.));
                for i in 0 .. mutual_relationships_count {
                    flow_network.add(FlowEdge::new(*x, women[i], 1.));
                }
            });

            Data { source, target, flow_network }
        }

        #[test]
        fn test() {
            for men_women_count in 1 .. 10 {
                let mut data = build_data(men_women_count, 1);
                assert_eq!(data.source, men_women_count * 2);
                assert_eq!(data.target, men_women_count * 2 + 1);
                let ff = assert_res(FordFulkerson::new(&mut data.flow_network, data.source, data.target));
                assert!(ff.marked[data.source]);
                assert!(!ff.marked[.. data.source].iter().all(|x| *x));
                assert_eq!(ff.maxflow, 1.);
            }
        }

    }

    #[test]
    fn closure_problem() {
    }
}
