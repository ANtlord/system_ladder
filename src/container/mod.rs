mod rbtree;
mod heap;
mod btree;

pub use rbtree::Tree;
pub use btree::Btree;
pub use heap::Heap;
pub use heap::IndexedHeap;
use crate::list::List;

// FIXME: make List able to provide iter_mut method
pub struct Bag<T> {
    inner: List<T>,
}

impl<T> Bag<T> {
    pub fn push(&mut self, v: T) {
        self.inner.push_back(v);
    }

    pub fn iter(&self) -> impl Iterator<Item=&T> {
        self.inner.iter()
    }
}

impl<T> Default for Bag<T> {
    fn default() -> Self {
        Self { inner: List::new() }
    }
}

pub struct Stack<T> {
    inner: List<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self { inner: List::new() }
    }

    pub fn push(&mut self, v: T) {
        self.inner.push_back(v);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop_back()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
