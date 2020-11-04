use crate::container::Bag;

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
    fn new(from: usize, to: usize, capacity: f64) -> Self {
        Self { from, to, flow: 0., capacity }
    }

    fn other(&self, than: usize) -> Result<usize, String> {
        if than == self.from {
            Ok(self.to)
        } else if than == self.to {
            Ok(self.from)
        } else {
            invalid_vertex(self, than)
        }
    }

    fn residual_capacity_to(&self, vertex: usize) -> Result<f64, String> {
        if vertex == self.from {
            Ok(self.flow)
        } else if vertex == self.to {
            Ok(self.capacity - self.flow)
        } else {
            invalid_vertex(self, vertex)
        }
    }

    fn add_residual_flow_to(&mut self, vertex: usize, delta: f64) -> Result<(), String> {
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

impl FlowNetwork {
    fn new(vertex_count: usize) -> Self {
        let mut edges = Vec::with_capacity(vertex_count);
        (0 .. vertex_count).for_each(|_| edges.push(Bag::default()));
        Self { edges, edge_store: Vec::new() }
    }

    fn add(&mut self, edge: FlowEdge) {
        let FlowEdge { from, to, .. } = edge;
        self.edge_store.push(edge);
        self.edges[from].push(self.edge_store.len() - 1);
        self.edges[to].push(self.edge_store.len() - 1);
    }

    fn adj(&self, vertex: usize) -> impl Iterator<Item=&FlowEdge> {
        self.edges[vertex].iter().map(move |x| &self.edge_store[*x])
    }

    fn len(&self) -> usize {
        self.edges.len()
    }
}
