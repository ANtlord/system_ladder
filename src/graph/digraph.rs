use std::marker::PhantomData;
use std::slice::Iter as SliceIter;
use crate::container::Bag;

#[cfg_attr(test, derive(Clone, Debug))]
#[derive(PartialEq)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub weight: f32,
}

pub struct Digraph {
    vertex_count: usize,
    edge_count: usize,
    data: Vec<Bag<Edge>>,
}

struct EdgeIter<'a> {
    data: &'a Vec<Bag<Edge>>,
}

impl Digraph {
    pub fn new(vertex_count: usize) -> Self {
        let mut data = Vec::with_capacity(vertex_count);
        (0 .. vertex_count).for_each(|_| data.push(Bag::default()));
        Self { data, vertex_count, edge_count: 0 }
    }

    pub fn add(&mut self, e: Edge) {
        self.data[e.from].push(e);
        self.edge_count += 1;
    }

    pub fn adj(&self, vertex: usize) -> impl Iterator<Item=&Edge> {
        self.data[vertex].iter()
    }

    pub fn len(&self) -> usize {
        self.vertex_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    #[test]
    fn adj() {
        let mut dig = Digraph::new(3);
        let expected_edges = vec![
            Edge{from: 0, to: 1, weight: 0.123},
            Edge{from: 0, to: 2, weight: 0.1},
            Edge{from: 0, to: 2, weight: 0.2},
            Edge{from: 0, to: 0, weight: 0.3},           
        ];

        expected_edges.iter().for_each(|x| dig.add(x.clone()));
        let from0 = dig.adj(0).collect::<Vec<&Edge>>();
        assert_eq!(from0, expected_edges.iter().collect::<Vec<&Edge>>());
    }
}
