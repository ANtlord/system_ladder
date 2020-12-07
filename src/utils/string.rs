use std::ptr::null_mut;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;
use std::ptr::NonNull;
use std::marker::Copy;
use std::mem;
use crate::tprintln;

/// Ukkonen's algorithm
struct SuffixTree<'a> {
    root: Node<'a>,
    to: Rc<RefCell<usize>>,
}

// pub struct Link<'a>(Cell<*mut Node<'a>>);
#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Debug))]
pub struct Link<'a>(Option<NonNull<Node<'a>>>);

impl<'a> From<&Node<'a>> for Link<'a> {
    fn from(node: &Node<'a>) -> Self {
        Link(Some(NonNull::from(node)))
    }
}

impl<'a> Link<'a> {
    fn set_suffix(&mut self, to: Link<'a>) {
        self.0.map(|mut last_node| unsafe { last_node.as_mut().suffix_link = to; });
    }
}

impl<'a> Default for Link<'a> {
    fn default() -> Self {
        // Link(Cell::new(null_mut()))
        Link(None)
    }
}

struct Child<'a>(Option<Box<Node<'a>>>);

impl<'a> Child<'a> {
    fn new(data: &'a str, from: usize, to: Weak<RefCell<usize>>) -> Self {
        Self(Some(Node::boxed(data, from, to)))
    }
}

impl<'a> From<Node<'a>> for Child<'a> {
    fn from(n: Node<'a>) -> Self {
        Self(Some(Box::new(n)))
    }
}

#[cfg_attr(test, derive(Clone))]
enum Position {
    Ptr(Weak<RefCell<usize>>),
    Data(usize),
}

struct Table<'a>([Child<'a>; 256]);

impl<'a> Table<'a> {
    fn put(&mut self, key: u8, value: Child<'a>) {
        self.0[key as usize] = value;
    }
}

// use std::ops::Index;
// use std::ops::IndexMut;
// 
// impl<'a> Index<u8> for Table<'a> {
//     type Output = Child<'a>;
// 
//     fn index(&self, index: u8) -> &Self::Output {
//         &self.0[index as usize]
//     }
// }
// 
// impl<'a> IndexMut<u8> for Table<'a> {
//     fn index_mut(&mut self, index: u8) -> &mut Self::Output {
//         &mut self.0[index as usize]
//     }
// }

struct Node<'a> {
    data: &'a str,
    from: usize,
    to: Position,
    nodes: [Child<'a>; 256],
    suffix_link: Link<'a>,
}

impl<'a> Node<'a> {
    fn new(data: &'a str, from: usize, to: Weak<RefCell<usize>>) -> Self {
        Self {data, from, to: Position::Ptr(to), nodes: make_children(), suffix_link: Link::default()}
    }

    fn inner(data: &'a str, from: usize, to: usize) -> Self {
        Self {data, from, to: Position::Data(to), nodes: make_children(), suffix_link: Link::default()}
    }

    fn boxed(data: &'a str, from: usize, to: Weak<RefCell<usize>>) -> Box<Self> {
        Box::new(Self::new(data, from, to))
    }

    fn len(&self) -> usize {
        let to = match &self.to {
            Position::Data(x) => *x,
            Position::Ptr(weak) => match weak.upgrade() {
                Some(x) => x.borrow().clone(),
                _ => panic!(),
            }
        };

        to - self.from
    }
}

fn make_children<'a>() -> [Child<'a>; 256] {
    unsafe {
        let mut res = mem::MaybeUninit::<[Child; 256]>::uninit();
        for i in 0 .. 256 {
            let mut ptr = res.as_mut_ptr() as *mut Child;
            ptr.add(i).write(Child(None));
        }

        res.assume_init()
    }
}

// fn next<'b, 'a: 'b>(node: &'b mut Node<'a>, child_key: usize) -> &'b mut Node<'a> {
//     let ptr = node as *mut Node;
// 
//     unsafe {
//         if let Some(ref mut inner) = (*ptr).nodes.get_mut(child_key).unwrap().0 {
//             inner
//         } else {
//             mem::transmute(ptr)
//         }
//     }
// }
// 
// struct ActivePoint<'a> {
//     node: &'a mut Node<'a>,
//     edge: Option<u8>,
//     length: usize,
//     root: Box<Node<'a>>,
// }
// 
// impl<'a> ActivePoint<'a> {
//     fn new(data: &'a str) -> Self {
//         let mut root = Box::new(Node::new(data, 0, Weak::new()));
//         Self {
//             node: unsafe {
//                 mem::transmute(root.as_mut() as *mut _)
//             },
//             edge: None,
//             length: 0,
//             root,
//         }
//     }
// 
//     fn next(&mut self, child_key: usize, remainder: usize, byte: u8, root: &'node mut Node<'a>) -> bool {
//         let nodeptr = self.node as *mut Node;
// 
//         unsafe {
//             if let Some(ref mut node) = (*nodeptr).nodes[child_key].0 {
//                 if remainder == node.len() {
//                     self.edge = None;
//                     self.length = 0;
//                     self.node = node;
//                 } else {
//                     self.edge = self.edge.or(Some(byte));
//                     self.length += 1;
//                     self.node = root;
//                 }
//             }
//         }
// 
//         false
//     }
// }

fn split_node<'a>(
    node: Node<'a>,
    input: &'a str,
    active_length: usize,
    current_end: Rc<RefCell<usize>>,
) -> Node<'a> {
    let current_end_ptr = current_end.borrow();
    let from = node.from;
    let mut new_node = Node::inner(input, from, from + active_length);
    let key = from + active_length;
    new_node.nodes[input.as_bytes()[key] as usize] = Child::new(
        input, key, Rc::downgrade(&current_end),
    );

    new_node.nodes[input.as_bytes()[*current_end_ptr] as usize] = Child::new(
        input, *current_end_ptr, Rc::downgrade(&current_end),
    );

    new_node
}

impl<'a> SuffixTree<'a> {
    fn new<T: AsRef<str> + ?Sized>(input: &'a T) -> Self {
        let current_end = Rc::new(RefCell::new(0usize));
        let mut root = Node::new(input.as_ref(), 0, Weak::new());
        let rootptr = &root as *const _;
        let mut count = 0;
        let mut unwritted_node_count = 0;

        // active point {
        // let active_node_ref = RefCell::new(&mut root);
        let mut active_node = &mut root;
        let mut active_edge: Option<u8> = None;
        let mut active_length = 0;
        // }
        //

        let mut remainder = 1;
        for byte in input.as_ref().as_bytes().iter() {
            let child_key = *byte as usize;
            let mut last_created_node = Link::default();
            if let Some(ref mut node) = active_node.nodes[child_key].0 {
                 if remainder == node.len() {
                     active_edge = None;
                     active_length = 0;
                     active_node = node;
                 } else {
                     active_edge = active_edge.or(Some(*byte));
                     active_length += 1;
                     active_node = &mut root;
                 };

                 remainder += 1;
                 unwritted_node_count += 1;
            } else {
                let current_end_ptr = current_end.borrow();
                while remainder > 1 {
                    let input = input.as_ref();
                    let active_edge_key = active_edge.unwrap() as usize;
                    let old_node = *active_node.nodes[active_edge_key].0.take().unwrap();
                    let new_node = split_node(old_node, input, active_length, current_end.clone());
                    active_node.nodes[active_edge_key] = new_node.into();

                    let ref inserted_node = active_node.nodes[active_edge_key].0;
                    let inserted_node_link = Link(inserted_node.as_ref().map(|x| x.as_ref().into()));
                    last_created_node.set_suffix(inserted_node_link);
                    last_created_node = inserted_node_link;
                    if active_node as *const _ == rootptr {
                        active_length -= 1;
                        active_edge = Some(input.as_bytes()[*current_end_ptr - active_length]);
                    } else {
                        active_node = match active_node.suffix_link.0 {
                            Some(ref mut x) => unsafe { x.as_mut() },
                            None => &mut root,
                        }
                    }

                    remainder -= 1;
                }

                active_node.nodes[child_key] = Child::new(input.as_ref(), count, Rc::downgrade(&current_end));
                active_edge = None;
                unwritted_node_count = 0;
            }

            let mut current_end_ptr = current_end.borrow_mut();
            *current_end_ptr = *current_end_ptr + 1;
            count += 1;
        }

        // dbg!(&current_end, unwritted_node_count);
        Self{root, to: current_end}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    // impl<'a> Clone for Node<'a> {
    //     fn clone(&self) -> Self {
    //         Self {
    //             data: self.data,
    //             from: self.from,
    //             to: self.to,
    //             nodes: self.nodes.clone(),
    //             suffix_link: self.suffix_link.clone(),
    //         }
    //     }
    // }

    impl<'a> fmt::Debug for Node<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Node")
                .field("data", &self.data)
                .field("from", &self.from)
                .field("to", &self.to)
                .field("suffix_link", &self.suffix_link)
                .finish()
        }
    }

    fn clone_leaf<'a>(from: &Child<'a>) -> Child<'a> {
        Child(from.0.as_ref().map(|from| {
            Box::new(Node {
                data: from.data,
                from: from.from,
                to: from.to.clone(),
                nodes: make_children(),
                suffix_link: Link::default(),
            })
        }))
        
    }

    struct NodesBuild<'a> {
        nodes: [Child<'a>; 256],
    }

    impl<'a> NodesBuild<'a> {
        fn new() -> Self {
            Self { nodes: make_children() }
        }

        // fn add(&mut self, key: char, from: usize, to: Position, suffix_link: Link<'a>, nodes: [Child<'a>; 256]) -> &mut Self {
        fn add(mut self, key: char, value: Node<'a>) -> Self {
            self.nodes[key as usize] = value.into();
            self
        }

        fn run(self) -> [Child<'a>; 256] {
            self.nodes
        }
    }


    impl PartialEq for Position {
        fn eq(&self, other: &Self) -> bool {
            if let (Position::Data(left), Position::Data(right)) = (self, other) {
                return left == right;
            }

            if let (Position::Ptr(left), Position::Ptr(right)) = (self, other) {
                if let (Some(left), Some(right)) = (left.upgrade(), right.upgrade()) {
                    return left == right;
                }
            }

            return false;
        }
    }

    impl<'a> PartialEq for Link<'a> {
        fn eq(&self, other: &Self) -> bool {
            if let (Some(left), Some(right)) = (self.0, other.0) {
                unsafe {
                    return left.as_ref() == right.as_ref();
                }
            }

            (None, None) == (self.0, other.0)
        }
    }

    impl fmt::Debug for Position {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Position::Ptr(weak) => {
                    let value = match weak.upgrade() {
                        Some(x) => format!("{}", x.borrow()),
                        None => "dangling".to_owned(),
                    };

                    f.write_fmt(format_args!("Position(Weak({}))", value.as_str()))
                }
                Position::Data(x) => f.write_fmt(format_args!("Position(Data({}))", x)),
            }
        }
    }

    impl<'a> PartialEq for Node<'a> {
        fn eq(&self, other: &Self) -> bool {
            let res = self.data == other.data;
            let res = res && self.from == other.from;
            let res = res && self.to == other.to;
            res && self.suffix_link == other.suffix_link
        }
    }

    #[test]
    fn no_repeats() {
        let data = "abc";
        let tree = SuffixTree::new(data);
        let expected_endptr = Rc::new(RefCell::new(data.len()));
        let expected_tree = SuffixTree{
            root: Node {
                data,
                from: 0,
                to: Position::Ptr(Weak::new()),
                nodes: NodesBuild::new()
                    .add('a', Node{
                        data,
                        from: 0,
                        to: Position::Ptr(Rc::downgrade(&expected_endptr)),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('b', Node{
                        data,
                        from: 1,
                        to: Position::Ptr(Rc::downgrade(&expected_endptr)),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('c', Node{
                        data,
                        from: 2,
                        to: Position::Ptr(Rc::downgrade(&expected_endptr)),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .run(),
                suffix_link: Link::default(),
            },
            to: Rc::new(RefCell::new(data.len())),
        };

        for i in 0 .. 256 {
            let key = i as u8;
            match key as char {
                'a' | 'b' | 'c' => assert_eq!(
                    tree.root.nodes[i].0, expected_tree.root.nodes[i].0, "key: {}", key as char,
                ),
                _ => assert!(tree.root.nodes[i].0.is_none(), "key = {}", key as char),
            }
        }
    }

    fn get_validated_node<'b, 'a: 'b>(node: &'a Node<'b>, key: u8, expected_from: usize, expected_to: usize) -> Option<&'a Node<'b>> {
        node.nodes[key as usize].0.as_ref().map(|node| {
            assert_eq!(node.from, expected_from);
            match node.to {
                Position::Data(to) => assert_eq!(to, expected_to),
                _ => assert!(false),
            }
            node.as_ref()
        })
    }

    fn test_nodes(actual: &[Child; 256], expected: &[Child; 256]) {
        for i in 0 .. 256 {
            let key = i as u8;
            assert_eq!(actual[i].0, expected[i].0, "key: {}", key as char);
        }
    }

    #[test]
    fn two_repeats() {
        let data = "abcabx";
        let tree = SuffixTree::new(data);
        let expected_endptr = Rc::new(RefCell::new(data.len()));
        let anode_nodes = NodesBuild::new()
            .add('c', Node{
                data,
                from: 2,
                to: Position::Ptr(Rc::downgrade(&expected_endptr)),
                suffix_link: Link::default(),
                nodes: NodesBuild::new().run(),
            })
            .add('x', Node{
                data,
                from: 5, // TODO: what's a number correct????
                to: Position::Ptr(Rc::downgrade(&expected_endptr)),
                suffix_link: Link::default(),
                nodes: NodesBuild::new().run(),
            })
            .run();

        let mut bnode_nodes = make_children();
        (0 .. 256).for_each(|i| bnode_nodes[i] = clone_leaf(&anode_nodes[i]));

        let mut expected_tree = SuffixTree{
            root: Node {
                data,
                from: 0,
                to: Position::Ptr(Weak::new()),
                nodes: NodesBuild::new()
                    .add('a', Node{
                        data,
                        from: 0,
                        to: Position::Data(2),
                        suffix_link: Link::default(),
                        nodes: anode_nodes,
                    })
                    .add('b', Node{
                        data,
                        from: 1,
                        to: Position::Data(2),
                        suffix_link: Link::default(),
                        nodes: bnode_nodes,
                    })
                    .add('c', Node{
                        data,
                        from: 2,
                        to: Position::Ptr(Rc::downgrade(&expected_endptr)),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('x', Node{
                        data,
                        from: 5,
                        to: Position::Ptr(Rc::downgrade(&expected_endptr)),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .run(),
                suffix_link: Link::default(),
            },
            to: Rc::new(RefCell::new(data.len())),
        };

        {
            let anode: &mut Node = unsafe {
                mem::transmute(expected_tree.root.nodes[b'a' as usize].0.as_mut().unwrap().as_mut() as *mut _)
            };

            let bnode: &Node = expected_tree.root.nodes[b'b' as usize].0.as_ref().unwrap().as_ref();
            anode.suffix_link = bnode.into();
        }

        test_nodes(&tree.root.nodes, &expected_tree.root.nodes);
        test_nodes(
            &tree.root.nodes[b'a' as usize].0.as_ref().unwrap().as_ref().nodes,
            &expected_tree.root.nodes[b'a' as usize].0.as_ref().unwrap().as_ref().nodes,
        );
    }

    #[test]
    fn three_repeats() {
        let data = "abcabxabcd";
        let tree = SuffixTree::new(data);
        let expected_endptr = Rc::new(RefCell::new(10));
        let expected_tree = SuffixTree{
            root: Node {
                data,
                from: 0,
                to: Position::Ptr(Weak::new()),
                nodes: NodesBuild::new()
                    .add('a', Node{
                        data,
                        from: 0,
                        to: Position::Data(2),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('b', Node{
                        data,
                        from: 1,
                        to: Position::Data(2),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('c', Node{
                        data,
                        from: 1,
                        to: Position::Data(2),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('d', Node{
                        data,
                        from: 9,
                        to: Position::Ptr(Rc::downgrade(&expected_endptr)),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('x', Node{
                        data,
                        from: 5,
                        to: Position::Data(6),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .run(),
                suffix_link: Link::default(),
            },
            to: Rc::new(RefCell::new(data.len())),
        };

        for i in 0 .. 256 {
            let key = i as u8;
            match key as char {
                'a' | 'b' | 'c' | 'd' | 'x' => assert_eq!(
                    tree.root.nodes[i].0, expected_tree.root.nodes[i].0, "key: {}", key as char,
                ),
                _ => assert!(tree.root.nodes[i].0.is_none(), "key = {}", key as char),
            }
        }
    }

    #[test]
    fn experiment() {
    }
}
