use std::marker::PhantomData;
use crate::list::List;
use std::slice::Iter as SliceIter;

pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub weight: f32,
}

struct Bag<T> {
    inner: List<T>,
}

impl<T> Bag<T> {
    fn new() -> Self {
        Self { inner: List::new() }
    }

    fn push(&mut self, v: T) {
        self.inner.push_back(v);
    }

    fn iter(&self) -> impl Iterator<Item=&T> {
        self.inner.iter()
    }
}

pub struct Digraph {
    vertex_count: usize,
    edge_count: usize,
    data: Vec<Bag<Edge>>,
}

impl Digraph {
    pub fn new(vertex_count: usize) -> Self {
        let mut data = Vec::with_capacity(vertex_count);
        (0 .. vertex_count).for_each(|_| data.push(Bag::new()));
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

    impl Clone for Edge {
        fn clone(&self) -> Self {
            Self{from: self.from, to: self.to, weight: self.weight}
        }
    }

    impl fmt::Debug for Edge {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Edge {{ from: {}, to: {}, weight: {} }}", self.from, self.to, self.weight)
        }
    }

    impl PartialEq for Edge {
        fn eq(&self, other: &Self) -> bool {
            self.from == self.from && self.to == self.to && self.weight == self.weight
        }
    }

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
