use std::cmp::Ordering;
use std::mem;

pub struct Node<K, V> {
    pub parent: *mut Node<K, V>,
    pub data: Option<(K, V)>,
    pub edges: Vec<Box<Node<K, V>>>,
}

impl<K: Ord, V> Node<K, V> {
    pub fn biggest(p: *mut Self) -> Box<Self> {
        Box::new(Self {
            parent: p,
            data: None,
            edges: vec![],
        })
    }

    pub fn cmp(&self, key: &K) -> Ordering {
        match self.data {
            Some((ref k, _)) => k.cmp(&key),
            None => Ordering::Greater,
        }
    }

    pub fn insert(&mut self, mut node: Box<Node<K, V>>) {
        let key = node.data.as_ref().map(|x| &x.0).unwrap();
        let place = self.edges.binary_search_by(|x| x.cmp(key));
        let mut index = place.unwrap_err();
        for i in index .. self.edges.len() {
            mem::swap(&mut self.edges[i], &mut node); // every children may get wrong pointer.
        }

        self.edges.push(node);
    }
}
