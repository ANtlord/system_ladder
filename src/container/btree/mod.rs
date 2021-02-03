use std::collections::BTreeMap;
use std::mem;
use std::cmp::Ordering;
use std::ptr;
use std::borrow::Borrow;
use std::fmt;

mod node;
use self::node::Node;

pub struct Btree<K, V> {
    maxlen: usize,
    root: Box<Node<K, V>>,
}

struct Len(usize);

impl Len {
    fn from(value: usize) -> Option<Self> {
        if value < 4 || value % 2 != 0 {
            return None
        }

        return Some(Len(value))
    }
}

impl<K: Ord + fmt::Debug, V> Btree<K, V> {
    fn new(maxlen: Len) -> Self {
        let Len(maxlen) = maxlen;
        let mut root = Box::new(Node {
            parent: ptr::null_mut(),
            data: None,
            edges: Vec::with_capacity(maxlen),
        });

        let rootptr = root.as_mut() as *mut _;
        root.edges.push(Node::biggest(rootptr));
        Self { maxlen, root }
    }

    fn get<Q>(&self, key: Q) -> Option<&V>
        where Q: Borrow<K>
    {
        println!("get");
        match self.find(key.borrow()) {
            Found(node) => node.data.as_ref().map(|x| &x.1),
            GoDown(_, _) => None,
        }
    }

    fn set(&mut self, key: K, val: V) {
        match self.find(&key) {
            Found(x) => unsafe {
                let node: *mut Node<K, V> = mem::transmute(x);
                if let Some((_, ref mut v)) = (*node).data {
                    *v = val
                }

                unreachable!();
            }
            GoDown(x, mut index) => {
                if x.parent.is_null() {
                    unreachable!()
                }

                // in case when we return equals to length - 1 we could replace the last element
                // regardless if it's smaller or not.
                let mut node: &mut Node<K, V> = unsafe { &mut *(x.parent) };
                // index += (node.edges[index].cmp(&key) == Ordering::Less) as usize;

                let mut tempnode = Box::new(Node {
                    parent: x.parent,
                    data: Some((key, val)),
                    edges: vec!(),
                });

                for i in index .. node.edges.len() {
                    mem::swap(&mut node.edges[i], &mut tempnode);
                }

                node.edges.push(tempnode);
                self.balance(node);
            }
        }
    }

    fn balance(&mut self, mut node: &mut Node<K, V>) {
        debug_assert!(node.edges.len() <= self.maxlen);
        loop {
            if node.edges.len() < self.maxlen {
                return
            }

            let half = node.edges.len() / 2;
            let mut midnode = node.edges.swap_remove(half - 1);
            let mut new_sibling = Box::new(Node {
                parent: node.parent,
                data: midnode.data.take(),
                edges: Vec::with_capacity(self.maxlen),
            });

            let mut biggest = Node::biggest(new_sibling.as_mut() as *mut _);
            biggest.edges = midnode.edges;
            while node.edges.len() > half {
                new_sibling.edges.push(node.edges.swap_remove(0))
            }
            new_sibling.edges.push(biggest);

            // assert new_sibling.data.key > node.data.key

            if new_sibling.parent.is_null() { // this means that we've got the seconds root.
                let mut old_root = mem::replace(&mut self.root, Box::new(Node {
                    parent: ptr::null_mut(),
                    data: None,
                    edges: Vec::with_capacity(self.maxlen),
                }));

                old_root.parent = self.root.as_mut() as *mut _;
                new_sibling.parent = self.root.as_mut() as *mut _;
                self.root.edges.push(new_sibling);
                self.root.edges.push(old_root);
                return
            }

            let parent = unsafe { &mut *new_sibling.parent };
            parent.insert(new_sibling);
            node = parent;
        }
    }

    fn find<'a>(&'a self, key: &K) -> SearchResult<'a, K, V> {
        let mut node: &Node<K, V> = &self.root.as_ref();
        loop {
            match find(node, key) {
                Found(node) => return Found(node),
                GoDown(n, node_index) => {
                    if n.edges.len() == 0 { // it's a leaf
                        return GoDown(n, node_index);
                    }

                    node = n;
                    continue;
                }
            }
        }
    }
}

enum SearchResult<'a, K, V> {
    Found(&'a Node<K, V>),
    GoDown(&'a Node<K, V>, usize),
}

use SearchResult::*;

/// Returns index of the wanted element or index of the closest smaller element.
fn find<'a, K: Ord + fmt::Debug, V>(node: &'a Node<K, V>, key: &K) -> SearchResult<'a, K, V> {
    match node.edges.binary_search_by(|x| x.cmp(key)) {
        Ok(x) => Found(&node.edges[x]),
        Err(mut x) => GoDown(&node.edges[x], x),
    }
}

mod tests {
    use super::*;
    #[test]
    fn bin() {
        let vals = vec![1,3,5,7,9];
        let hay = 2;
        let (mut left, mut right) = (0, vals.len() - 1);
        let mut index = 0;
        while left <= right {
            index = (left + right) / 2;
            if vals[index] > hay {
                right = index - 1;
            } else if vals[index] < hay {
                left = index + 1;
            } else {
                break;
            }
        }

        assert_eq!(index, 1);
    }

    #[test]
    fn outbounds() {
        let mut vals = Vec::with_capacity(2);
        vals.push(10);
        let sl = vals.as_slice();
        unsafe {
            let p = sl.get_unchecked(1);
        }
    }

    #[derive(Eq, PartialEq, Debug)]
    struct Sword {
        damage: u32,
        kind: u8,
        level: u8,
    }

    use std::error::Error;
    #[test]
    fn create() -> Result<(), Box<dyn Error>> {
        let len = Len::from(4).ok_or("lenght must be greater that 4")?;
        let mut tree = Btree::new(len);
        tree.set(10, Sword {
            damage: 8,
            kind: 1,
            level: 5,
        });
        Ok(())
    }

    mod set {
        use super::*;

        #[test]
        fn reverse_root() -> Result<(), Box<dyn Error>> {
            let len = Len::from(4).ok_or("lenght must be greater that 4")?;
            let mut tree = Btree::new(len);
            tree.set(10, Sword { damage: 8, kind: 1, level: 5 });
            tree.set(5, Sword { damage: 6, kind: 1, level: 3 });
            let actual_root_keys: Vec<Option<u8>> = tree.root.edges.iter()
                .map(|x| x.data.as_ref().map(|(k, _)| *k)).collect();
            assert_eq!(actual_root_keys, [Some(5), Some(10), None]);
            Ok(())
        }

        #[test]
        fn obverse_root() -> Result<(), Box<dyn Error>> {
            let len = Len::from(4).ok_or("lenght must be greater that 4")?;
            let mut tree = Btree::new(len);
            tree.set(5, Sword { damage: 6, kind: 1, level: 3 });
            tree.set(10, Sword { damage: 8, kind: 1, level: 5 });
            let actual_root_keys: Vec<Option<u8>> = tree.root.edges.iter()
                .map(|x| x.data.as_ref().map(|(k, _)| *k)).collect();
            assert_eq!(actual_root_keys, [Some(5), Some(10), None]);
            Ok(())
        }

        fn select<'a, K, V>(tree: &'a Btree<K, V>, path: &[usize]) -> &'a Node<K, V> {
            let mut node = tree.root.as_ref();
            for index in path {
                node = node.edges[*index].as_ref();
            }

            node
        }

        #[test]
        fn overflow_root() -> Result<(), Box<dyn Error>> {
            let len = Len::from(4).ok_or("lenght must be greater that 4")?;
            let mut tree = Btree::new(len);
            tree.set(10, Sword { damage: 6, kind: 1, level: 3 });
            tree.set(20, Sword { damage: 8, kind: 1, level: 5 });
            tree.set(30, Sword { damage: 2, kind: 1, level: 1 });
            assert_eq!(collect_keys(&tree.root), [Some(20), None]);
            assert_eq!(collect_keys(select(&tree, &[0])), [Some(10), None]);
            assert_eq!(collect_keys(select(&tree, &[1])), [Some(30), None]);

            Ok(())
        }

        #[test]
        fn overflow_leaf() -> Result<(), Box<dyn Error>> {
            let len = Len::from(4).ok_or("lenght must be greater that 4")?;
            let mut tree = Btree::new(len);
            tree.set(10, Sword { damage: 6, kind: 1, level: 3 });
            tree.set(20, Sword { damage: 8, kind: 1, level: 5 });
            tree.set(30, Sword { damage: 2, kind: 1, level: 1 });
            tree.set(40, Sword { damage: 1, kind: 1, level: 1 });
            tree.set(50, Sword { damage: 9, kind: 1, level: 5 });
            assert_eq!(collect_keys(&tree.root), [Some(20), Some(40), None]);
            assert_eq!(collect_keys(select(&tree, &[0])), [Some(10), None]);
            assert_eq!(collect_keys(select(&tree, &[1])), [Some(30), None]);
            assert_eq!(collect_keys(select(&tree, &[2])), [Some(50), None]);
            Ok(())
        }

        fn collect_keys<K: Copy, V>(node: &Node<K, V>) -> Vec<Option<K>> {
            node.edges.iter().map(|x| x.data.as_ref().map(|(k, _)| *k)).collect::<Vec<_>>()
        }

        #[test]
        fn overflow_inner() -> Result<(), Box<dyn Error>> {
            let len = Len::from(4).ok_or("lenght must be greater that 4")?;
            let mut tree = Btree::new(len);
            tree.set(10, Sword { damage: 6, kind: 1, level: 3 });
            tree.set(20, Sword { damage: 8, kind: 1, level: 5 });
            tree.set(30, Sword { damage: 2, kind: 1, level: 1 });
            tree.set(40, Sword { damage: 1, kind: 1, level: 1 });
            tree.set(50, Sword { damage: 9, kind: 1, level: 5 });
            tree.set(60, Sword { damage: 9, kind: 2, level: 2 });
            tree.set(70, Sword { damage: 4, kind: 2, level: 1 });
            assert_eq!(collect_keys(&tree.root), [Some(40), None]);
            assert_eq!(collect_keys(select(&tree, &[0])), [Some(20), None]);
            assert_eq!(collect_keys(select(&tree, &[1])), [Some(60), None]);
            assert_eq!(collect_keys(select(&tree, &[0, 0])), [Some(10), None]);
            assert_eq!(collect_keys(select(&tree, &[0, 1])), [Some(30), None]);
            assert_eq!(collect_keys(select(&tree, &[1, 0])), [Some(50), None]);
            assert_eq!(collect_keys(select(&tree, &[1, 1])), [Some(70), None]);
            Ok(())
        }

        #[test]
        fn insert_1000() -> Result<(), Box<dyn Error>> {
            let len = Len::from(4).ok_or("lenght must be greater that 4")?;
            let mut tree = Btree::new(len);
            let size = 10000;
            for i in 0 .. size {
                tree.set(i, i * 10);
            }

            for i in 0 .. size {
                let res = tree.get(i).ok_or(format!("can't find {}", i))?;
                assert_eq!(*res, i * 10);
            }

            Ok(())
        }
    }

    mod get {
        use super::*;
        #[test]
        fn overflow_root() -> Result<(), Box<dyn Error>> {
            let len = Len::from(4).ok_or("lenght must be greater that 4")?;
            let mut tree = Btree::new(len);
            tree.set(10, Sword { damage: 6, kind: 1, level: 3 });
            tree.set(20, Sword { damage: 8, kind: 1, level: 5 });
            tree.set(30, Sword { damage: 2, kind: 1, level: 1 });
            
            assert_eq!(
                tree.get(10).ok_or("can't get 10th sword")?,
                &Sword { damage: 6, kind: 1, level: 3 },
            );
            assert_eq!(
                tree.get(20).ok_or("can't get 20th sword")?,
                &Sword { damage: 8, kind: 1, level: 5 },
            );
            assert_eq!(
                tree.get(30).ok_or("can't get 30th sword")?,
                &Sword { damage: 2, kind: 1, level: 1 },
            );

            Ok(())
        }
    }
}
