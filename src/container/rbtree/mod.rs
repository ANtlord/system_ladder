mod node;

use std::cmp::Ord;
use std::cmp::Ordering;
use std::fmt;
use std::mem;
use std::ptr::NonNull;
use std::collections::BTreeMap;

use self::node::Node;
use self::node::NodePtr;

// TODO: exchange its left child and right child. In this case, a node can be justified red or
// black according to if its right child is larger than its left child. That could safe some
// memory.
// TODO: modify it to left leaning red-black tree
// http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.139.282&rep=rep1&type=pdf
// TODO: implement Drop
pub struct Tree<T, P> {
    root: NodePtr<T, P>,
}

impl<T: Ord, P> Tree<T, P> {
    fn allocate(&self, value: T) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(value))) }
    }

    fn unallocate(&self, value: NonNull<T>) {
        unsafe { Box::from_raw(value.as_ptr()); }
    }

    fn insert(&mut self, key: T, value: P) {
        match self.root {
            Some(mut x) => unsafe {
                let node = x.as_mut().add(self.allocate(key), value);
                node::repair(node);
                let mut parent = node.as_ref().parent;
                while let Some(p) = parent {
                    self.root = parent;
                    parent = p.as_ref().parent;
                }
            },
            None => self.root = Some(Node::head(self.allocate(key), value)),
        }
    }

    fn get(&self, value: &T) -> Option<&P> {
        let node = self.find(value)?;
        unsafe {
            Some(&(*(&node.as_ref().payload as *const _)))
        }
    }

    fn find(&self, value: &T) -> NodePtr<T, P> {
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

    unsafe fn drop_node(&mut self, mut node_ptr: NonNull<Node<T, P>>) -> P {
        let node_ptr = node_ptr.as_mut().del();
        self.unallocate(node_ptr.as_ref().value);
        let node = *Box::from_raw(node_ptr.as_ptr());
        return node.payload;
    }

    fn del(&mut self, value: &T) -> Option<P> {
        let mut root_ptr = self.root.take()?;
        unsafe {
            if root_ptr.as_ref().left.or(root_ptr.as_ref().right).is_none() {
                return Some(self.drop_node(root_ptr));
            }
        }

        self.root = Some(root_ptr);
        if let Some(mut x) = self.find(value) {
            return Some(unsafe {self.drop_node(x)});
        }

        return None;
    }
}

impl<T: Ord, P> Default for Tree<T, P> {
    fn default() -> Self {
        Self { root: None }
    }
}

#[cfg(test)]
mod tests {
    use super::Node as BaseNode;
    use super::*;
    use std::cell::RefCell;

    trait Visitor<'a, T, P> {
        fn every(&self, _: &'a Node<T, P>) {}
        fn left(&self, _: &'a Node<T, P>) {}
        fn right(&self, _: &'a Node<T, P>) {}
        fn leaf(&self, _: &'a Node<T, P>) {}
    }

    struct LeafCollector<'a, T, P> {
        data: RefCell<Vec<&'a Node<T, P>>>,
    }

    impl<'a, T, P> LeafCollector<'a, T, P> {
        fn new() -> Self {
            let data = RefCell::new(vec![]);
            Self { data }
        }
    }

    impl<'a, T, P> Visitor<'a, T, P> for LeafCollector<'a, T, P> {
        fn leaf(&self, node: &'a Node<T, P>) {
            self.data.borrow_mut().push(node);
        }
    }

    struct RedCollector<'a, T, P> {
        data: RefCell<Vec<&'a Node<T, P>>>,
    }

    impl<'a, T, P> RedCollector<'a, T, P> {
        fn new() -> Self {
            let data = RefCell::new(vec![]);
            Self { data }
        }
    }

    impl<'a, T, P> Visitor<'a, T, P> for RedCollector<'a, T, P> {
        fn every(&self, node: &'a Node<T, P>) {
            if node.color.is_red() {
                self.data.borrow_mut().push(node);
            }
        }
    }

    unsafe fn extend_lifetime<'a, 'b, T>(r: &'b T) -> &'a T {
        mem::transmute::<&'b T, &'a T>(r)
    }

    fn recurse_check<'a, T: Ord + fmt::Debug + 'a, P: 'a>(
        ptr: NonNull<Node<T, P>>,
        level: u8,
        visitor: &impl Visitor<'a, T, P>,
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

    fn make_tree(count: u8) -> Tree<u8, ()> {
        let mut tree = Tree::default();
        for i in 0..count {
            tree.insert(i, ());
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

            let leaf_collector = LeafCollector::new();
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
        for i in 1..7 {
            let tree = make_tree(i);
            let red_node_collector = RedCollector::new();
            recurse_check(tree.root.unwrap(), 0, &red_node_collector);
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

    mod delete {
        use super::*;
        use std::rc::Rc;
        use crate::random::xorshift_rng as random;

        struct Model {
            id: usize,
            drop_indicators: Rc<RefCell<Vec<bool>>>,
        }

        impl Drop for Model {
            fn drop(&mut self) {
                self.drop_indicators.borrow_mut()[self.id] = true;
            }
        }

        fn make_tree(drop_indicators: Rc<RefCell<Vec<bool>>>) -> Tree<usize, Model> {
            let mut tree = Tree::default();
            for i in 0..drop_indicators.borrow().len() {
                tree.insert(i, Model{
                    id: i,
                    drop_indicators: drop_indicators.clone(),
                });
            }

            tree
        }

        #[test]
        fn basic() {
            //     __b3__
            //    /      \
            //   b1       b5
            //  / \      /  \
            // b0  b2   b4  r7
            //             /  \
            //            b6   b8
            //                   \
            //                   r9
            let mut drop_indicators = Rc::new(RefCell::new(vec![false; 10]));
            let mut tree = make_tree(drop_indicators.clone());
            let red_node_collector = RedCollector::new();
            recurse_check(tree.root.unwrap(), 0, &red_node_collector);
            for node in red_node_collector.data.borrow().iter() {
                let val = unsafe { node.value.as_ref() };
                assert!([7, 9].contains(val), "unexpected value {}", val);
            }

            let node = tree.find(&3).unwrap();
            let root = tree.root.unwrap();
            assert_eq!(node, root);
            drop(tree.del(&3));
            dbg!(&drop_indicators);
            let drop_indicators = drop_indicators.borrow();
            assert!(!drop_indicators[..3].iter().fold(true, |x, y| x && *y));
            assert!(drop_indicators[3]);
            assert!(!drop_indicators[4..].iter().fold(true, |x, y| x && *y));
            assert!(tree.get(&3).is_none());
        }

        #[test]
        fn single() {
            let count = 1;
            let mut drop_indicators = Rc::new(RefCell::new(vec![false; count]));
            let mut tree = make_tree(drop_indicators.clone());
            tree.del(&0);
        }

        #[test]
        fn both() {
            let count = 2;
            let mut drop_indicators = Rc::new(RefCell::new(vec![false; count]));
            let mut tree = make_tree(drop_indicators.clone());
            unsafe {
                assert_eq!(tree.find(&0).unwrap().as_ref().value.as_ref(), &0);
                assert_eq!(tree.find(&1).unwrap().as_ref().value.as_ref(), &1);
            }

            let Model{ id, .. } = tree.del(&0).unwrap();
            assert_eq!(id, 0);
            assert!(drop_indicators.borrow()[0]);
            unsafe {
                assert_eq!(tree.root.unwrap().as_ref().value.as_ref(), &1);
                assert_eq!(tree.find(&1).unwrap().as_ref().value.as_ref(), &1);
            }

            let Model{ id, .. } = tree.del(&1).unwrap();
            assert_eq!(id, 1);
            assert!(drop_indicators.borrow()[1]);
        }

        #[test]
        fn test10() {
            return;
            let count = 4;
            let mut drop_indicators = Rc::new(RefCell::new(vec![false; count]));
            let mut tree = make_tree(drop_indicators.clone());
            for i in 0 .. count {
                unsafe {
                    assert_eq!(tree.find(&i).unwrap().as_ref().value.as_ref(), &i);
                    assert_eq!(tree.find(&i).unwrap().as_ref().payload.id, i);
                }
            }

            unsafe {
                assert_eq!(tree.root.unwrap().as_ref().value.as_ref(), &1);
            }

            let id = tree.del(&0).map(|x| x.id);
            assert_eq!(id, Some(0));
            unsafe {
                assert_eq!(tree.root.unwrap().as_ref().value.as_ref(), &2);
            }

            let id = tree.del(&1).map(|x| x.id);
            assert_eq!(id, Some(1));
            unsafe {
                assert_eq!(tree.root.unwrap().as_ref().value.as_ref(), &2);
            }


            // let id = tree.del(&2).map(|x| x.id);
            // assert_eq!(id, Some(2));

            // for j in i .. count {
            //     unsafe {
            //         assert_eq!(tree.find(&j).map(|x| x.as_ref().value.as_ref().clone()), Some(j));
            //         assert_eq!(tree.find(&j).unwrap().as_ref().payload.id, j);
            //     }
            // }


        }
    }
}
