mod node;

use std::cmp::Ord;
use std::fmt;
use std::mem;
use std::ptr::NonNull;

use self::node::Node;
use self::node::NodePtr;

// TODO: exchange its left child and right child. In this case, a node can be justified red or
// black according to if its right child is larger than its left child. That could safe some
// memory.
// TODO: modify it to left leaning red-black tree
// http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.139.282&rep=rep1&type=pdf
pub struct Tree<T> {
    root: NodePtr<T>,
}

impl<T: Ord> Tree<T> {
    fn allocate(&self, value: T) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(value))) }
    }

    fn unallocate(&self, value: NonNull<T>) {
        unsafe { Box::from_raw(value.as_ptr()); }
    }

    fn add(&mut self, value: T) {
        match self.root {
            Some(mut x) => unsafe {
                let node = x.as_mut().add(self.allocate(value));
                node::repair(node);
                dbg!(node);
                let mut parent = node.as_ref().parent;
                while let Some(p) = parent {
                    self.root = parent;
                    parent = p.as_ref().parent;
                }
            },
            None => self.root = Some(Node::head(self.allocate(value))),
        }
    }

    fn find(&self, value: &T) -> NodePtr<T> {
        let mut node = self.root;
        while let Some(current) = node {
            let current_val = unsafe { current.as_ref().value.as_ref() };
            if current_val == value {
                return node;
            } else if current_val > value {
                node = unsafe { current.as_ref().left };
            } else {
                node = unsafe { current.as_ref().right };
            }
        }

        node
    }

    fn del(&mut self, value: &T) {
        let root = self.root;
        if root.is_none() {
            return;
        }

        let mut root = root.unwrap();
        let root_ref = unsafe { root.as_mut() };
        let mut node = match self.find(value) {
            Some(x) => x,
            None => return,
        };

        if root_ref.left.or(root_ref.right).is_none() {
            let val = unsafe { root_ref.del() };
            self.unallocate(val);
            self.root = None;
        }

        let val = unsafe { node.as_mut().del() };
        self.unallocate(val);
    }
}

impl<T: Ord> Default for Tree<T> {
    fn default() -> Self {
        Self { root: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    trait Visitor<'a, T> {
        fn every(&self, _: &'a Node<T>) {}
        fn left(&self, _: &'a Node<T>) {}
        fn right(&self, _: &'a Node<T>) {}
        fn leaf(&self, _: &'a Node<T>) {}
    }

    struct LeafCollector<'a, T> {
        data: RefCell<Vec<&'a Node<T>>>,
    }

    impl<'a, T> LeafCollector<'a, T> {
        fn new() -> Self {
            let data = RefCell::new(vec![]);
            Self { data }
        }
    }

    impl<'a, T> Visitor<'a, T> for LeafCollector<'a, T> {
        fn leaf(&self, node: &'a Node<T>) {
            self.data.borrow_mut().push(node);
        }
    }

    struct RedCollector<'a, T> {
        data: RefCell<Vec<&'a Node<T>>>,
    }

    impl<'a, T> RedCollector<'a, T> {
        fn new() -> Self {
            let data = RefCell::new(vec![]);
            Self { data }
        }
    }

    impl<'a, T> Visitor<'a, T> for RedCollector<'a, T> {
        fn every(&self, node: &'a Node<T>) {
            if node.color.is_red() {
                self.data.borrow_mut().push(node);
            }
        }
    }

    unsafe fn extend_lifetime<'a, 'b, T>(r: &'b T) -> &'a T {
        mem::transmute::<&'b T, &'a T>(r)
    }

    fn recurse_check<'a, T: Ord + fmt::Debug + 'a>(
        ptr: NonNull<Node<T>>,
        level: u8,
        visitor: &impl Visitor<'a, T>,
    ) {
        unsafe {
            if level > 0 {
                dbg!(
                    &ptr.as_ref().value,
                    &ptr.as_ref().parent.unwrap().as_ref().value
                );
            }

            visitor.every(extend_lifetime(ptr.as_ref()));
            if let Some(x) = ptr.as_ref().left {
                visitor.left(extend_lifetime(x.as_ref()));
                recurse_check(x, level + 1, visitor);
            }

            if let Some(x) = ptr.as_ref().right {
                visitor.right(extend_lifetime(x.as_ref()));
                recurse_check(x, level + 1, visitor);
            }

            if ptr.as_ref().left.or(ptr.as_ref().right).is_none() {
                visitor.leaf(extend_lifetime(ptr.as_ref()));
            }
        }
    }

    fn make_tree(count: u8) -> Tree<u8> {
        let mut tree = Tree::default();
        for i in 0..count {
            tree.add(i);
        }
        tree
    }

    #[test]
    fn equal_number_of_blacks_from_root_to_every_leaf() {
        for i in 1..255 {
            let tree = make_tree(i);
            let black_node_counter_pivot = {
                let mut node = tree.root;
                let mut black_node_counter = 0;
                while let Some(current_node) = node {
                    let node_ref = unsafe { current_node.as_ref() };
                    if node_ref.color.is_black() {
                        black_node_counter += 1;
                    }

                    node = node_ref.left;
                }

                black_node_counter
            };

            let leaf_collector: LeafCollector<u8> = LeafCollector::new();
            recurse_check(tree.root.unwrap(), 0, &leaf_collector);
            for e in leaf_collector.data.borrow().iter() {
                let mut black_node_counter = if e.color.is_black() { 1 } else { 0 };
                let mut parent = e.parent;
                while let Some(x) = parent {
                    let xref = unsafe { x.as_ref() };
                    if xref.color.is_black() {
                        black_node_counter += 1
                    }

                    parent = xref.parent;
                }

                assert_eq!(
                    black_node_counter, black_node_counter_pivot,
                    "one of branches has incorrect number of black nodes"
                );
            }
        }
    }

    #[test]
    fn red_node_has_black_children_only() {
        for i in 1..255 {
            let tree = make_tree(i);
            let red_node_collector: RedCollector<u8> = RedCollector::new();
            for e in red_node_collector.data.borrow().iter() {
                if e.left.or(e.right).is_some() {
                    unsafe {
                        assert!(e.left.unwrap().as_ref().color.is_red());
                        assert!(e.right.unwrap().as_ref().color.is_red());
                    }
                }
            }
        }
    }

    #[test]
    fn black_root() {
        for i in 1..255 {
            let tree = make_tree(i);
            unsafe {
                assert!(
                    tree.root.unwrap().as_ref().color.is_black(),
                    "head root is not black for {} nodes", i
                );
            }
        }
    }

    #[test]
    fn delete() {
        //     __b3__
        //    /      \
        //   b1       b5
        //  / \      /  \
        // b0  b2   b4  r7
        //             /  \
        //            b6   b8
        //                   \
        //                   r9
        let mut tree = make_tree(10);
        let red_node_collector: RedCollector<u8> = RedCollector::new();
        for node in red_node_collector.data.borrow().iter() {
            let val = unsafe { node.value.as_ref() };
            assert!([7, 9].contains(val), "unexpected value {}", val);
        }



    }
}
