use std::ptr::null_mut;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;
use std::ptr::NonNull;
use std::marker::Copy;
use std::mem;
use std::fmt;
use crate::tprintln;

const END: u8 = b'0';

static LOREM_IPSUM: &str = r#"
Lorem Ipsum is simply dummy text of the printing and typesetting industry.
"#;

// Lorem Ipsum has been the industry's standard dummy text ever since the 1500s,
// when an unknown printer took a galley of type and scrambled it to make a type specimen book.
/// Ukkonen's algorithm
struct SuffixTree<'a> {
    root: Node<'a>,
    to: Rc<RefCell<usize>>,
}

// pub struct Link<'a>(Cell<*mut Node<'a>>);
#[derive(Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
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

const ROOT_TO: usize = 0;

struct Node<'a> {
    data: &'a str,
    from: usize,
    to: usize, // 0 for root
    nodes: [Child<'a>; 256],
    suffix_link: Link<'a>,
}

impl<'a> Node<'a> {
    fn new(data: &'a str, from: usize, to: Weak<RefCell<usize>>) -> Self {
        let to = *to.upgrade().unwrap().borrow();
        //debug_assert_ne!(from, to);
        Self {data, from, to, nodes: make_children(), suffix_link: Link::default()}
    }

    fn inner(data: &'a str, from: usize, to: usize) -> Self {
        Self {data, from, to, nodes: make_children(), suffix_link: Link::default()}
    }

    fn boxed(data: &'a str, from: usize, to: Weak<RefCell<usize>>) -> Box<Self> {
        Box::new(Self::new(data, from, to))
    }

    fn len(&self) -> usize {
        self.finished_at() - self.from
    }

    fn finished_at(&self) -> usize {
        self.to
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

struct ActivePoint<'a> {
    node: NonNull<Node<'a>>,
    edge: Option<u8>,
    length: usize,
    root: Box<Node<'a>>,
}


impl<'a> ActivePoint<'a> {
    fn new(data: &'a str) -> Self {
        let root_to_ptr = Rc::new(RefCell::new(ROOT_TO));
        let mut root = Box::new(Node::new(data, 0, Rc::downgrade(&root_to_ptr)));
        Self { node: NonNull::from(root.as_ref()), edge: None, length: 0, root }
    }

    fn has_child(&self, key: u8, current_end: usize) -> bool {
        if key == END {
            return false;
        }

        let data_bytes = self.root.data.as_bytes();
        let key = self.edge.unwrap_or(key);
        let noderef = unsafe { self.node.as_ref() };
        // return noderef.nodes[key as usize].0.is_some();
        if let Some(ref child) = noderef.nodes[key as usize].0 {
            return data_bytes[child.from + self.length] == data_bytes[current_end];
        }

        return false;
    }

    fn update(&mut self, key: u8) {
        self.edge = self.edge.or(Some(key));
        self.length += 1;
    }

    // move further if the active point is in the end of the its edge.
    fn try_follow_edge(&mut self) {
        // dbg!(self.root.as_ref() as *const _, &self.root.nodes[b'a' as usize].0);
        let key = self.edge.take().unwrap();
        let noderef = unsafe { self.node.as_mut() };
        let node = noderef.nodes[key as usize].0.as_ref().unwrap();
        if self.length == node.len() {
            self.length = 0;
            dbg!(&noderef, node, key as char);
            self.node = NonNull::from(node.as_ref());
        } else {
            self.edge = Some(key);
        }
        // dbg!(self.root.as_ref() as *const _, &self.root.nodes[b'a' as usize].0);
    }

    fn add_node(&mut self, for_letter: u8, current_end: Rc<RefCell<usize>>) -> &Node<'a> {
        dbg!(for_letter as char);
        let mut noderef = unsafe { self.node.as_mut() };
        let loc = *current_end.borrow();
        noderef.nodes[for_letter as usize] = Child::new(
            self.root.data, loc, Rc::downgrade(&Rc::new(RefCell::new(self.root.data.len()))),
        );

        noderef
    }

    // `at` points to the node to split.
    fn split_node(&mut self, at: u8, for_letter: u8, current_end: Rc<RefCell<usize>>) -> &Node<'a> {
        let at = at as usize;
        let old_node = {
            let mut noderef = unsafe { self.node.as_mut() };
            dbg!(at as u8 as char, noderef as *const _, &noderef.nodes[at].0, &self.root);
            noderef.nodes[at].0.take().unwrap()
        };

        let new_node = self._split_node(*old_node, for_letter, current_end);
        let mut noderef = unsafe { self.node.as_mut() };
        noderef.nodes[at] = new_node.into();
        noderef.nodes[at].0.as_ref().unwrap()
    }

    fn _split_node(&self, node: Node<'a>, for_letter: u8, current_end: Rc<RefCell<usize>>) -> Node<'a> {
        let input = self.root.data;
        let active_length = self.length;
        // if node.from == 0 {
        //     dbg!(&node, active_length);
        // }

        let to = node.finished_at();
        debug_assert!(active_length < to);
        let current_end_ptr = current_end.borrow();
        let from = node.from;
        let mut new_node = Node::inner(input, from, from + active_length);
        let key = from + active_length;
        let end = Rc::new(RefCell::new(self.root.data.len()));
        new_node.nodes[input.as_bytes()[key] as usize] = Child::new(input, key, Rc::downgrade(&end));
        new_node.nodes[for_letter as usize] = Child::new(input, *current_end_ptr, Rc::downgrade(&end));
        new_node
    }

    fn follow_suffix(&mut self) {
        let noderef = unsafe { self.node.as_ref() };
        dbg!(&noderef);
        self.node = match noderef.suffix_link.0 {
            Some(ref x) => x.clone(),
            None => NonNull::from(self.root.as_ref()),
        };
    }

    fn is_at_root(&self) -> bool {
        self.root.as_ref() as *const _ == self.node.as_ptr() as * const _
    }
}

impl<'a> SuffixTree<'a> {
    fn new<T: AsRef<str> + ?Sized>(input: &'a T) -> Self {
        let current_end = Rc::new(RefCell::new(0usize));
        let mut unwritted_node_count = 0;
        let mut active_point = ActivePoint::new(input.as_ref());
        let mut remainder = 1;
        for (i, byte) in input.as_ref().as_bytes().iter().chain(&[END]).enumerate() {
            *current_end.borrow_mut() = i;
            let mut last_created_node = Link::default();
            dbg!(*byte as char);
            if dbg!(active_point.has_child(*byte, *current_end.borrow())) {
                active_point.update(*byte);
                active_point.try_follow_edge();
                remainder += 1;
                continue;
            }

            while remainder > 1 {
                // debug_assert!(active_point.length > 0);
                // dbg!(*active_point.edge.as_ref().unwrap() as char);
                let input = input.as_ref();
                let inserted_node_link = match active_point.edge {
                    Some(key) => active_point.split_node(key, *byte, current_end.clone()),
                    _ => active_point.add_node(*byte, current_end.clone()),
                }.into();

                // r2
                last_created_node.set_suffix(inserted_node_link);

                last_created_node = inserted_node_link;
                if dbg!(active_point.is_at_root()) { // r1
                    dbg!(active_point.length, remainder);
                    active_point.length -= 1;
                    let current_end_ptr = current_end.borrow();
                    let loc = *current_end_ptr - active_point.length;
                    active_point.edge = Some(if input.len() == loc {
                        END
                    } else {
                        input.as_bytes()[loc]
                    });
                } else { // r3
                    active_point.follow_suffix();
                }

                remainder -= 1;
                // dbg!(*active_point.edge.as_ref().unwrap() as char);
            }

            active_point.add_node(*byte, current_end.clone());
            active_point.edge = None;
        }

        // *current_end.borrow_mut() += 1;
        Self{root: *active_point.root, to: current_end}
    }

    fn find(&self, pattern: &str) -> Option<usize> {
        let mut count = 0;
        let mut pattern_iter = pattern.as_bytes().iter();
        let mut node: &Node = &self.root.nodes.get(*pattern_iter.next()? as usize)?.0.as_ref()?.as_ref();
        dbg!(&self.root.data[node.from .. node.finished_at()]);
        let mut node_byte_count = 1;
        for byte in pattern_iter {
            let mut text_index = node.from + node_byte_count;
            if text_index >= node.finished_at() {
                node = node.nodes.get(*byte as usize)?.0.as_ref()?;
                node_byte_count = 0;
                text_index = node.from;
            }

            if dbg!(self.root.data.as_bytes()[text_index]) != dbg!(*byte) {
                dbg!(node, &self.root.data[node.from .. node.finished_at()]);
                return None;
            }

            node_byte_count += 1;
        }

        Some(node.from + node_byte_count - pattern.len())
    }
}

#[cfg(debug_assertions)]
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

#[cfg(debug_assertions)]
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
                    return dbg!(left.as_ref()) == dbg!(right.as_ref());
                }
            }

            (None, None) == (self.0, other.0)
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

    fn end_node(data: &str, expected_endptr: Rc<RefCell<usize>>) -> Node {
        Node {
            data,
            from: data.len(),
            to: data.len(),
            suffix_link: Link::default(),
            nodes: NodesBuild::new().run(),
        }
    }

    fn test_nodes(actual: &[Child; 256], expected: &[Child; 256]) {
        for i in 0 .. 256 {
            let key = i as u8;
            assert_eq!(actual[i].0, expected[i].0, "key: {}", key as char);
        }
    }

    fn select_ref<'b, 'a: 'b>(nodes: &'b [Child<'a>; 256], path: &[u8]) -> &'b Node<'a> {
        let ret = select(&nodes, path);
        unsafe { mem::transmute(ret) }
    }

    fn select<'b, 'a: 'b>(nodes: &'b [Child<'a>; 256], path: &[u8]) -> *const Node<'a> {
        let mut count = 0;
        let mut nodes = nodes;
        loop {
            let key = path[count];
            let ret = &nodes[key as usize];
            if count + 1 == path.len() {
                let n: &Node = ret.0.as_ref().unwrap().as_ref().into();
                break n as *const _;
            } else {
                count += 1;
                nodes = &ret.0.as_ref().expect(&format!("key `{}` is None", key as char)).nodes;
            }
        }
    }

    fn link_inner_nodes(nodes: &mut [Child; 256], from: &[u8], to: &[u8]) {
        let (from_ptr, to_ptr) = (select(&nodes, from), select(&nodes, to));
        let to_node: &Node = unsafe { mem::transmute(to_ptr) };
        let from_node: &mut Node = unsafe { mem::transmute(from_ptr) };
        from_node.suffix_link = to_node.into();
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
                to: ROOT_TO,
                nodes: NodesBuild::new()
                    .add(END as char, end_node(data, expected_endptr.clone()))
                    .add('a', Node{
                        data,
                        from: 0,
                        to: data.len(),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('b', Node{
                        data,
                        from: 1,
                        to: data.len(),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('c', Node{
                        data,
                        from: 2,
                        to: data.len(),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .run(),
                suffix_link: Link::default(),
            },
            to: Rc::new(RefCell::new(data.len())),
        };

        test_nodes(&tree.root.nodes, &expected_tree.root.nodes);
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
                to: data.len(),
                suffix_link: Link::default(),
                nodes: NodesBuild::new().run(),
            })
            .add('x', Node{
                data,
                from: 5, // TODO: what's a number correct (by design)????
                to: data.len(),
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
                to: ROOT_TO,
                nodes: NodesBuild::new()
                    .add(END as char, end_node(data, expected_endptr.clone()))
                    .add('a', Node{
                        data,
                        from: 0,
                        to: 2,
                        suffix_link: Link::default(),
                        nodes: anode_nodes,
                    })
                    .add('b', Node{
                        data,
                        from: 1,
                        to: 2,
                        suffix_link: Link::default(),
                        nodes: bnode_nodes,
                    })
                    .add('c', Node{
                        data,
                        from: 2,
                        to: data.len(),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('x', Node{
                        data,
                        from: 5,
                        to: data.len(),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .run(),
                suffix_link: Link::default(),
            },
            to: Rc::new(RefCell::new(data.len())),
        };

        {
            let bnode: Link = expected_tree.root.nodes[b'b' as usize].0.as_ref().unwrap().as_ref().into();
            let anode: &mut Node = expected_tree.root.nodes[b'a' as usize].0.as_mut().unwrap();
            anode.suffix_link = bnode.into();
        }

        test_nodes(&tree.root.nodes, &expected_tree.root.nodes);
        test_nodes(
            &tree.root.nodes[b'a' as usize].0.as_ref().unwrap().as_ref().nodes,
            &expected_tree.root.nodes[b'a' as usize].0.as_ref().unwrap().as_ref().nodes,
        );

        test_nodes(
            &tree.root.nodes[b'b' as usize].0.as_ref().unwrap().as_ref().nodes,
            &expected_tree.root.nodes[b'b' as usize].0.as_ref().unwrap().as_ref().nodes,
        );
    }

    impl<'a> From<&Child<'a>> for Link<'a> {
        fn from(ch: &Child<'a>) -> Self {
            ch.0.as_ref().unwrap().as_ref().into()
        }
    }

    // impl<'a> Child<'a> {
    //     fn suffix_link_ref(&self) -> &Node {
    //         unsafe {
    //             self.0.as_ref().unwrap().suffix_link.0.as_ref().unwrap().as_ref()
    //         }
    //     }
    // }

    #[test]
    fn three_repeats() {
        let data = "abcabxabcd";
        let tree = SuffixTree::new(data);
        let expected_endptr = Rc::new(RefCell::new(10));
        let cnode_nodes = NodesBuild::new()
            .add('a', Node{
                data,
                from: 3,
                to: data.len(),
                suffix_link: Link::default(),
                nodes: NodesBuild::new().run(),
            })
            .add('d', Node{
                data,
                from: 9,
                to: data.len(),
                suffix_link: Link::default(),
                nodes: NodesBuild::new().run(),
            })
            .run();

        let mut b_cnode_nodes = make_children();
        (0 .. 256).for_each(|i| b_cnode_nodes[i] = clone_leaf(&cnode_nodes[i]));

        let mut ab_cnode_nodes = make_children();
        (0 .. 256).for_each(|i| ab_cnode_nodes[i] = clone_leaf(&cnode_nodes[i]));

        let bnode_nodes = NodesBuild::new()
            .add('c', Node{
                data,
                from: 2,
                to: 3,
                suffix_link: Link::default(),
                nodes: b_cnode_nodes,
            })
            .add('x', Node{
                data,
                from: 5, // TODO: what's a number correct (by design)????
                to: data.len(),
                suffix_link: Link::default(),
                nodes: NodesBuild::new().run(),
            })
            .run();

        let anode_nodes = NodesBuild::new()
            .add('c', Node{
                data,
                from: 2,
                to: 3,
                suffix_link: Link::default(),
                nodes: ab_cnode_nodes,
            })
            .add('x', Node{
                data,
                from: 5, // TODO: what's a number correct (by design)????
                to: data.len(),
                suffix_link: Link::default(),
                nodes: NodesBuild::new().run(),
            })
            .run();

        let mut expected_tree = SuffixTree{
            root: Node {
                data,
                from: 0,
                to: ROOT_TO,
                nodes: NodesBuild::new()
                    .add(END as char, end_node(data, expected_endptr.clone()))
                    .add('a', Node{
                        data,
                        from: 0,
                        to: 2,
                        suffix_link: Link::default(),
                        nodes: anode_nodes,
                    })
                    .add('b', Node{
                        data,
                        from: 1,
                        to: 2,
                        suffix_link: Link::default(),
                        nodes: bnode_nodes,
                    })
                    .add('c', Node{
                        data,
                        from: 2,
                        to: 3,
                        suffix_link: Link::default(),
                        nodes: cnode_nodes,
                    })
                    .add('d', Node{
                        data,
                        from: 9,
                        to: data.len(),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('x', Node{
                        data,
                        from: 5,
                        to: data.len(),
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .run(),
                suffix_link: Link::default(),
            },
            to: Rc::new(RefCell::new(data.len())),
        };

        link_inner_nodes(&mut expected_tree.root.nodes, &[b'a'], &[b'b']);
        link_inner_nodes(&mut expected_tree.root.nodes, &[b'a', b'c'], &[b'b', b'c']);
        link_inner_nodes(&mut expected_tree.root.nodes, &[b'b', b'c'], &[b'c']);

        test_nodes(&tree.root.nodes, &expected_tree.root.nodes);

        let actual_anodes = &tree.root.nodes[b'a' as usize].0.as_ref().unwrap().nodes;
        let expected_anodes = &expected_tree.root.nodes[b'a' as usize].0.as_ref().unwrap().nodes;
        test_nodes(actual_anodes, expected_anodes);

        let actual_bnodes = &tree.root.nodes[b'b' as usize].0.as_ref().unwrap().nodes;
        let expected_bnodes = &expected_tree.root.nodes[b'b' as usize].0.as_ref().unwrap().nodes;
        test_nodes(&actual_bnodes, &expected_bnodes);

        let actual_cnodes = &tree.root.nodes[b'c' as usize].0.as_ref().unwrap().nodes;
        let expected_cnodes = &expected_tree.root.nodes[b'c' as usize].0.as_ref().unwrap().nodes;
        test_nodes(&actual_cnodes, &expected_cnodes);

        let actual_dnodes = &tree.root.nodes[b'd' as usize].0.as_ref().unwrap().nodes;
        let expected_dnodes = &expected_tree.root.nodes[b'd' as usize].0.as_ref().unwrap().nodes;
        test_nodes(&actual_dnodes, &expected_dnodes);

        let actual_xnodes = &tree.root.nodes[b'x' as usize].0.as_ref().unwrap().nodes;
        let expected_xnodes = &expected_tree.root.nodes[b'x' as usize].0.as_ref().unwrap().nodes;
        test_nodes(&actual_xnodes, &expected_xnodes);

        test_nodes(
            &select_ref(&tree.root.nodes, &[b'a', b'c']).nodes,
            &select_ref(&expected_tree.root.nodes, &[b'a', b'c']).nodes,
        );

        test_nodes(
            &select_ref(&tree.root.nodes, &[b'b', b'c']).nodes,
            &select_ref(&expected_tree.root.nodes, &[b'b', b'c']).nodes,
        );
    }

    mod inner_extension {
        use super::*;

        #[test]
        fn without_suffix() {
            let data = "abcadak";
            let tree = SuffixTree::new(data);
            let expected_endptr = Rc::new(RefCell::new(7));

            let mut expected_tree = SuffixTree{
                root: Node {
                    data,
                    from: 0,
                    to: ROOT_TO,
                    nodes: NodesBuild::new()
                        .add(END as char, end_node(data, expected_endptr.clone()))
                        .add('a', Node{
                            data,
                            from: 0,
                            to: 1,
                            suffix_link: Link::default(),
                            nodes: NodesBuild::new()
                                .add('k', Node{
                                    data,
                                    from: 6,
                                    to: data.len(),
                                    nodes: make_children(),
                                    suffix_link: Link::default(),
                                })
                                .add('d', Node{
                                    data,
                                    from: 4,
                                    to: data.len(),
                                    nodes: make_children(),
                                    suffix_link: Link::default(),
                                })
                                .add('b', Node{
                                    data,
                                    from: 1,
                                    to: data.len(),
                                    nodes: make_children(),
                                    suffix_link: Link::default(),
                                })
                                .run(),
                        })
                        .add('b', Node{
                            data,
                            from: 1,
                            to: data.len(),
                            suffix_link: Link::default(),
                            nodes: make_children(),
                        })
                        .add('c', Node{
                            data,
                            from: 2,
                            to: data.len(),
                            suffix_link: Link::default(),
                            nodes: make_children(),
                        })
                        .add('d', Node{
                            data,
                            from: 4,
                            to: data.len(),
                            suffix_link: Link::default(),
                            nodes: make_children(),
                        })
                        .add('k', Node{
                            data,
                            from: 6,
                            to: data.len(),
                            suffix_link: Link::default(),
                            nodes: make_children(),
                        })
                        .run(),
                    suffix_link: Link::default(),
                },
                to: expected_endptr,
            };

            test_nodes(&tree.root.nodes, &expected_tree.root.nodes);
        }

        #[test]
        fn with_suffix() {
        }
    }

    #[test]
    fn undefined_repeat() {
        let data = "abab";
        let tree = SuffixTree::new(data);
        let expected_endptr = Rc::new(RefCell::new(data.len()));
        let mut expected_tree = SuffixTree{
            root: Node {
                data,
                from: 0,
                to: ROOT_TO,
                nodes: NodesBuild::new()
                    .add(END as char, end_node(data, expected_endptr.clone()))
                    .add('a', Node{
                        data,
                        from: 0,
                        to: 2,
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new()
                            .add('a', Node{
                                data,
                                from: 2,
                                to: data.len(),
                                suffix_link: Link::default(),
                                nodes: make_children(),
                            })
                            .add(END as char, Node{
                                data,
                                from: 4,
                                to: data.len(),
                                suffix_link: Link::default(),
                                nodes: make_children(),
                            }).run(),
                    })
                    .add('b', Node{
                        data,
                        from: 1,
                        to: 2,
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new()
                            .add('a', Node{
                                data,
                                from: 2,
                                to: data.len(),
                                suffix_link: Link::default(),
                                nodes: make_children(),
                            })
                            .add(END as char, Node{
                                data,
                                from: 4,
                                to: data.len(),
                                suffix_link: Link::default(),
                                nodes: make_children(),
                            }).run(),
                    })
                    .run(),
                suffix_link: Link::default(),
            },
            to: Rc::new(RefCell::new(data.len())),
        };

        link_inner_nodes(&mut expected_tree.root.nodes, &[b'a'], &[b'b']);
        test_nodes(&tree.root.nodes, &expected_tree.root.nodes);
        test_nodes(
            &select_ref(&tree.root.nodes, &[b'a']).nodes,
            &select_ref(&expected_tree.root.nodes, &[b'a']).nodes,
        );

        test_nodes(
            &select_ref(&tree.root.nodes, &[b'b']).nodes,
            &select_ref(&expected_tree.root.nodes, &[b'b']).nodes,
        );
    }

    // mod find {
    //     use super::*;

    //     #[test]
    //     fn true_positive(){
    //         let data = "dd";
    //         let tree = SuffixTree::new(data);
    //         let expected_endptr = Rc::new(RefCell::new(data.len()));

    //         let mut expected_tree = SuffixTree{
    //             root: Node {
    //                 data,
    //                 from: 0,
    //                 to: Position::Ptr(Weak::new()),
    //                 nodes: NodesBuild::new()
    //                     .add(END as char, end_node(data, expected_endptr.clone()))
    //                     .add('d', Node{
    //                         data,
    //                         from: 0,
    //                         to: Position::Data(1),
    //                         suffix_link: Link::default(),
    //                         nodes: NodesBuild::new()
    //                             .add(END as char, end_node(data, expected_endptr.clone()))
    //                             .add('d', Node{
    //                                 data,
    //                                 from: 1,
    //                                 to: Position::Ptr(Rc::downgrade(&expected_endptr)),
    //                                 suffix_link: Link::default(),
    //                                 nodes: make_children(),
    //                             }).run(),
    //                     }).run(),
    //                 suffix_link: Link::default(),
    //             },
    //             to: expected_endptr,
    //         };
    //         //let mut expected_tree = SuffixTree{
    //         //    root: Node {
    //         //        data,
    //         //        from: 0,
    //         //        to: Position::Ptr(Weak::new()),
    //         //        nodes: NodesBuild::new()
    //         //            .add(END as char, end_node(data, expected_endptr.clone()))
    //         //            .add('d', Node{
    //         //                data,
    //         //                from: 0,
    //         //                to: Position::Ptr(Rc::downgrade(&expected_endptr)),
    //         //                suffix_link: Link::default(),
    //         //                nodes: make_children(),
    //         //            })
    //         //            .add('u', Node{
    //         //                data,
    //         //                from: 1,
    //         //                to: Position::Ptr(Rc::downgrade(&expected_endptr)),
    //         //                suffix_link: Link::default(),
    //         //                nodes: make_children(),
    //         //            })
    //         //            .add('m', Node {
    //         //                data,
    //         //                from: 2,
    //         //                to: Position::Data(3),
    //         //                suffix_link: Link::default(),
    //         //                nodes: NodesBuild::new()
    //         //                    // .add('m', Node {
    //         //                    // }),
    //         //                    // .add('y')
    //         //                    .run(),
    //         //            })
    //         //            .add('y', Node {
    //         //                data,
    //         //                from: 4,
    //         //                to: Position::Ptr(Rc::downgrade(&expected_endptr)),
    //         //                suffix_link: Link::default(),
    //         //                nodes: make_children(),
    //         //            })
    //         //            .run(),
    //         //        suffix_link: Link::default(),
    //         //    },
    //         //    to: Rc::new(RefCell::new(data.len())),
    //         //};

    //         test_nodes(&tree.root.nodes, &expected_tree.root.nodes);
    //         // let tree = SuffixTree::new("dummy and indus");
    //         // dbg!(b"dummy");
    //         // let position = tree.find("dum");
    //         // assert_eq!(position, Some(0));
    //     }

    //     #[test]
    //     fn true_negative(){}

    //     #[test]
    //     fn false_positive(){}

    //     #[test]
    //     fn false_negative(){}
    // }

    // #[test]
    // fn experiment() {
    // }
}
