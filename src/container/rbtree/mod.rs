mod node; 

use std::cmp::Ord;
use std::mem;
use std::ptr::NonNull;
use std::fmt;

use self::node::NodePtr;
use self::node::Node;

pub struct Tree<T> {
    head: NodePtr<T>,
}

impl<T: Ord> Tree<T> {
    fn new(value: T) -> Self {
        Self {
            head: Some(Node::head(value)),
        }
    }

    fn add(&mut self, value: T) {
        match self.head {
            Some(mut x) => unsafe {
                let node = x.as_mut().add(value);
                node::repair(node);
                dbg!(node);
                let mut parent = node.as_ref().parent;
                while let Some(p) = parent {
                    self.head = parent;
                    parent = p.as_ref().parent;
                }
            },
            None => self.head = Some(Node::head(value)),
        }
    }
}

impl<T: Ord> Default for Tree<T> {
    fn default() -> Self {
        Self { head: None }
    }
}

mod tests {
    use super::*;

    fn recurse_check<T: Ord + fmt::Debug>(ptr: NonNull<Node<T>>, level: u8) {
        unsafe {
            if level > 0 {
                // dbg!(ptr, ptr.as_ref().parent.unwrap().as_ref() as *const _);
                dbg!(&ptr.as_ref().value, &ptr.as_ref().parent.unwrap().as_ref().value);
            }

            if let Some(x) = ptr.as_ref().left {
                recurse_check(x, level + 1);
            }

            if let Some(x) = ptr.as_ref().right {
                recurse_check(x, level + 1);
            }
        }
    }

    #[test]
    fn basic() {
        let mut tree = Tree::new(0);
        for i in 1 .. 18 {
            tree.add(i);
        }
        recurse_check(tree.head.unwrap(), 0);

        assert!(false);
    }
}
