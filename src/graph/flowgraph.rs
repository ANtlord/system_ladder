use std::mem;

use crate::container::Bag;

#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub struct FlowEdge {
    from: usize,
    to: usize,
    flow: f64,
    capacity: f64,
}

fn invalid_vertex<T>(edge: &FlowEdge, vertex: usize) -> Result<T, String> {
    Err(format!("invalid vertex {} for Flow edge is from {} to {}", vertex, edge.from, edge.to))
}

impl FlowEdge {
    pub fn new(from: usize, to: usize, capacity: f64) -> Self {
        Self { from, to, flow: 0., capacity }
    }

    pub fn other(&self, than: usize) -> Result<usize, String> {
        if than == self.from {
            Ok(self.to)
        } else if than == self.to {
            Ok(self.from)
        } else {
            invalid_vertex(self, than)
        }
    }

    pub fn residual_capacity_to(&self, vertex: usize) -> Result<f64, String> {
        if vertex == self.from {
            Ok(self.flow)
        } else if vertex == self.to {
            Ok(self.capacity - self.flow)
        } else {
            invalid_vertex(self, vertex)
        }
    }

    pub fn add_residual_flow_to(&mut self, vertex: usize, delta: f64) -> Result<(), String> {
        if vertex == self.from {
            self.flow -= delta;
            Ok(())
        } else if vertex == self.to {
            self.flow += delta;
            Ok(())
        } else {
            invalid_vertex(self, vertex)
        }
    }
}

pub struct FlowNetwork {
    edges: Vec<Bag<usize>>,
    edge_store: Vec<FlowEdge>,
}

struct Iter<'a, T: Iterator<Item=&'a usize>> {
    edge_store: &'a mut [FlowEdge],
    indexes: T,
}

impl<'a, T: Iterator<Item=&'a usize>> Iterator for Iter<'a, T> {
    type Item = &'a mut FlowEdge;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.indexes.next()?;
        unsafe { mem::transmute(Some(&mut self.edge_store[*index])) }
    }
}

impl FlowNetwork {
    pub fn new(vertex_count: usize) -> Self {
        let mut edges = Vec::with_capacity(vertex_count);
        (0 .. vertex_count).for_each(|_| edges.push(Bag::default()));
        Self { edges, edge_store: Vec::new() }
    }

    pub fn add(&mut self, edge: FlowEdge) {
        let FlowEdge { from, to, .. } = edge;
        self.edge_store.push(edge);
        self.edges[from].push(self.edge_store.len() - 1);
        self.edges[to].push(self.edge_store.len() - 1);
    }

    pub fn edge(&self, index: usize) -> Option<&FlowEdge> {
        self.edge_store.get(index)
    }

    pub fn edge_mut(&mut self, index: usize) -> Option<&mut FlowEdge> {
        self.edge_store.get_mut(index)
    }

    pub fn adj(&self, vertex: usize) -> impl Iterator<Item=&usize> {
        self.edges[vertex].iter()
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }
}
