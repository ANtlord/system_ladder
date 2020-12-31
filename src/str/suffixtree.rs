use std::io;
use std::io::Write;
use std::ptr::null_mut;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;
use std::borrow::Borrow;
use std::ptr::NonNull;
use std::marker::Copy;
use std::mem;
use std::fmt;

use crate::tprintln;
use crate::trace::Trace;
use crate::container::Stack;

const END: u8 = b'\0';
const DELIMETER: u8 = 0x01; // SOH like in FIX protocol.
const ROOT_TO: usize = 0;

static mut TR: Option<Trace> = None;

/// print which passes through cache. It's very handy if a program finishes suddenly.
fn diprint(val: impl AsRef<str>) {
    let out = io::stdout();
    let mut l = out.lock();
    l.write(format!("{}\n", val.as_ref()).as_bytes());
    l.flush();
}

static LOREM_IPSUM: &str = include_str!("lorem_ipsum.txt");

struct SuffixTree<'a> {
    data: Vec<&'a str>,
    root: Box<Node>,
    word_count: usize,
    end: u8,
}

#[derive(Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Link(Option<NonNull<Node>>);

impl From<&Node> for Link {
    fn from(node: &Node) -> Self {
        Link(Some(NonNull::from(node)))
    }
}

impl Link {
    fn set_suffix(&mut self, to: Link) {
        if let Link(Some(mut node)) = self {
            unsafe { node.as_mut().suffix_link = to; }
        }
    }
}

impl Default for Link {
    fn default() -> Self {
        Link(None)
    }
}

struct Child(Option<Box<Node>>);

impl From<Node> for Child {
    fn from(n: Node) -> Self {
        Self(Some(Box::new(n)))
    }
}

struct Node {
    views: Vec<Option<(usize, usize)>>,
    nodes: [Child; 256],
    suffix_link: Link,
    markmask: u8, // 00 - root, 01 - appears in the first word only
}

impl Node {
    fn new(from: usize, to: usize) -> Self {
        Self {
            views: vec![Some((from, to))],
            nodes: make_children(),
            suffix_link: Link::default(),
            markmask: 1,
        }
    }

    fn boxed(from: usize, to: usize) -> Box<Self> {
        Box::new(Self::new(from, to))
    }

    fn len(&self, at: usize) -> usize {
        self.to(at) - self.from(at)
    }

    fn from(&self, at: usize) -> usize {
        self.views[at].unwrap().0
    }

    fn to(&self, at: usize) -> usize {
        self.views[at].unwrap().1
    }
}

fn make_children() -> [Child; 256] {
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
    data: &'a str,
    node: NonNull<Node>,
    edge: Option<u8>,
    length: usize,
    root: Box<Node>,
    end: u8,
    word_count: usize,
}

impl<'a> ActivePoint<'a> {
    fn new(data: &'a str, root: Box<Node>, end: u8, word_count: usize) -> Self {
        Self { data, node: NonNull::from(root.as_ref()), edge: None, length: 0, root, end, word_count }
    }

    /// Updates active length and links last node which was splitted or a new node created from.
    ///
    /// This is required for a case when the current was splitted before the `last_created_node`.
    /// See `post_suffix_linking` test.
    fn update(&mut self, last_created_node: &mut Link) {
        self.length += 1;
        let link = Link(Some(self.node));
        if !self.is_root(link) {
            last_created_node.set_suffix(link);
        }
    }

    /// Checks if the current node has a child at `key`.
    fn has_child(&self) -> bool {
        let key = self.edge.unwrap();
        // probable redudant check.
        if key == self.end {
            return false;
        }

        let data_bytes = self.data.as_bytes();
        let noderef = unsafe { self.node.as_ref() };
        noderef.nodes[key as usize].0.is_some()
    }

    fn is_root(&self, link: Link) -> bool {
        match link.0 {
            Some(x) => x.as_ptr() as *const _ == self.root.as_ref() as *const _,
            _ => false,
        }
    }

    fn is_prefix(&self, for_letter: u8) -> bool {
        let key = self.edge.unwrap();
        let noderef = unsafe { self.node.as_ref() };
        let node = noderef.nodes[key as usize].0.as_ref().unwrap();
        self.data.as_bytes()[self.length + node.from(self.word_count - 1)] == for_letter
    }

    /// Move further if the active length is in the end of the its edge. 
    /// 
    /// It happends when we need to insert a suffix longer than the suffix which the child
    /// node of the current one carries. 
    ///
    /// Moving means:
    /// - Reduce the active length by length of suffix of the proper child of the current node.
    /// - Change the active edge (which is actualy a symbol) to that one which the current suffix
    /// ends by.
    /// - Change the current active node to the child node.
    ///
    /// Render assets/str/suffixtree/split_node.dot. It shows how moving works and the reason why
    /// it's needed.
    fn try_follow_edge(&mut self) -> bool {
        let key = self.edge.take().unwrap();
        let noderef: &Node = unsafe { self.node.as_ref() };
        let node = noderef.nodes[key as usize].0.as_ref().unwrap();
        if self.length >= node.len(self.word_count - 1) {
            self.length -= node.len(self.word_count - 1);
            self.edge = Some(self.data.as_bytes()[node.to(self.word_count - 1)]);
            self.node = NonNull::from(node.as_ref());
            true
        } else {
            self.edge = Some(key);
            false
        }
    }

    /// Adds a child node to the current one at `at`.
    ///
    /// This method is invoked when the current node is inner node or it's root. The new child node
    /// carries suffix starts from `current_end`
    fn add_node(&mut self, current_end: usize) -> &Node {
        let at = self.edge.unwrap();
        let mut noderef = unsafe { self.node.as_mut() };
        let node = Node::new(current_end, self.data.len());
        noderef.nodes[at as usize] = node.into();
        noderef
    }

    /// Splits a child node at `at` of the current node.
    ///
    /// It replaces the old child node at `at` by the new one. The new child node carries suffix from the
    /// begginning of the old child node consists of as much letter as designed in `self.length`.
    ///
    /// The new child node has two child nodes (which are grandchild nodes for the current one).
    /// The first one carries suffix from end of the new child node to the end of the old child
    /// node. The suffix can the rest of the word.
    /// The second one carries suffix from the current handled letter to the end of the word.
    ///
    /// Render assets/str/suffixtree/split_node.dot to get the picture. Gray nodes don't make
    /// any sense they are just for consistence. Black nodes are affected nodes.
    fn split_node(&mut self, for_letter: u8, current_end: usize) -> &Node {
        let at = self.edge.unwrap() as usize;
        let old_node = {
            let mut noderef = unsafe { self.node.as_mut() };
            noderef.nodes[at].0.take().unwrap()
        };

        let new_node = self._split_node(old_node, for_letter, current_end);
        let mut noderef = unsafe { self.node.as_mut() };
        noderef.nodes[at] = new_node.into();
        noderef.nodes[at].0.as_ref().unwrap()
    }

    fn _split_node(&self, mut node: Box<Node>, for_letter: u8, current_end: usize) -> Node {
        let to = node.to(self.word_count - 1);
        debug_assert!(self.length < to);
        let from = node.from(self.word_count - 1);
        let divide_suffix_at = from + self.length;
        let mut new_node = Node::new(from, divide_suffix_at);
        node.views[self.word_count - 1] = Some((divide_suffix_at, node.to(self.word_count - 1)));
        new_node.nodes[self.data.as_bytes()[divide_suffix_at] as usize] = Child(Some(node));
        new_node.nodes[for_letter as usize] = Node::new(current_end, self.data.len()).into();
        new_node
    }

    fn follow_suffix(&mut self) {
        let noderef = unsafe { self.node.as_ref() };
        self.node = match noderef.suffix_link.0 {
            Some(ref x) => *x,
            None => NonNull::from(self.root.as_ref()),
        };
    }

    fn is_at_root(&self) -> bool {
        self.root.as_ref() as *const _ == self.node.as_ptr() as * const _
    }
}

impl<'a> SuffixTree<'a> {
    /// Ukkonen's algorithm is used to build a suffix tree.
    ///
    /// There are two loops. In the first one we go over bytes of input in the second one we insert
    /// new nodes as long as we have symbols to insert. `remainder` is the counter of the symbols. A
    /// node doesn't carry actual symbols but a string view (pair of pointers represents a piece of
    /// the input string). Every node has an array of 256 elements.
    ///
    /// There is also a help structure `active point` which has:
    /// - length: length of suffix to insert.
    /// - node: what node be the parent of the new child node.
    /// - edge: index which the new child node meant to be inserted.
    ///
    ///
    /// When a letter is processed (iterating over outer loop):
    ///
    /// - If current active node doesn't have a child at the index the active edge equals to then a
    /// new node inserted in the array of children.
    ///
    /// - If a node is not inserted then it means then its place already is used by another node.
    /// We need following the child node. If active length is equal or greater length of the active
    /// node's child at the index (it means length of the suffix the node carries) then we change
    /// current node by the child and reduce the active length by length of the active node.  Then
    /// go to the begginning of the list.
    ///
    /// - If the following fails then check if the current suffix is a prefix of the suffix of the
    /// current active node's child at the index the active edge equals to. NOTE that the suffix we
    /// need to insert is a string view from current position until current position + active
    /// length. Then we pick the NEXT letter.
    ///
    /// - If it's not a prefix then we split the child node which takes the place at the index the
    /// active edge equals to and which is meant to be for the current suffix. In fact the old node
    /// suffix shrinks from left by the active length. The new child node carries suffix from the
    /// position the old node carried before the shrinking to its new start after the shrinking.
    /// The new child node takes the place of the old node so the old node become a child of the
    /// new child node (become grandchild of the current active node). There is also one more node
    /// to insert which carries suffix from the position of the current letter is being processed
    /// until the end of input string. The node is a child of the new child node and a leaf as
    /// well. (Carries only one letter right after insertion.)
    ///
    ///
    /// The next is executed only if a node was inserted or splitted:
    ///
    /// - The previous inserted node links to the new one.
    ///
    /// - If the active node is the root active length is positive then decrement it change the
    /// active edge to the next unprocessed letter.
    ///
    /// - Otherwise change the active node to the node the inserted node has suffix link to or to
    /// the root.
    fn extend(self, input: &'a str) -> Self {
        let end = self.end;
        let mut root = self.root;
        let word_count = self.word_count + 1;
        root.markmask = 0b11;
        let mut active_point = ActivePoint::new(input, root, end, word_count);
        let mut remainder = 0; // how many nodes should be inserted.

        'outer: for (i, byte) in input.as_bytes().iter().chain(&[end]).enumerate() {
            let mut last_created_node = Link::default();
            remainder += 1;
            while remainder > 0 {
                debug_assert!(active_point.length <= remainder);
                if active_point.length == 0 {
                    active_point.edge = Some(*byte);
                }

                let inserted_node_link: Link = if !active_point.has_child() {
                    active_point.add_node(i).into()
                } else if active_point.try_follow_edge() {
                    continue;
                } else if active_point.is_prefix(*byte) {
                    active_point.update(&mut last_created_node);
                    continue 'outer;
                } else {
                    active_point.split_node(*byte, i).into()
                };

                if !active_point.is_root(inserted_node_link) {
                    last_created_node.set_suffix(inserted_node_link);
                    last_created_node = inserted_node_link;
                }

                remainder -= 1;
                if active_point.is_at_root() && active_point.length > 0 { // r1
                    active_point.length -= 1;
                    let next_byte_position_to_insert = i - remainder + 1;
                    active_point.edge = Some(if input.len() == next_byte_position_to_insert {
                        end
                    } else {
                        input.as_bytes()[next_byte_position_to_insert]
                    });

                } else { // r3
                    active_point.follow_suffix();
                }
            }
        }

        let mut data = self.data;
        data.push(input);
        Self {
            data,
            root: active_point.root,
            end: active_point.end,
            word_count,
        }
    }

    fn new<T: AsRef<str> + ?Sized>(input: &'a T, end: u8) -> Self {
        let mut root = Box::new(Node::new(0, ROOT_TO));
        root.markmask = 0b11;
        Self{root, end, word_count: 0, data: vec!()}.extend(input.as_ref())
    }

    // TODO: implement searching in several words
    fn find<Byte, I>(&self, mut pattern_iter: I) -> Option<usize>
        where
            Byte: Borrow<u8>,
            I: Iterator<Item=Byte>,
    {
        let mut node = self.root.nodes.get(*pattern_iter.next()?.borrow() as usize)?.0.as_ref()?;
        let mut node_byte_count = 1;
        let mut pattern_len = 1;
        for byte in pattern_iter {
            let mut text_index = node.from(0) + node_byte_count; // bug
            if text_index >= node.to(0) { // bug
                node = node.nodes.get(*byte.borrow() as usize)?.0.as_ref()?;
                node_byte_count = 0;
                text_index = node.from(0); // bug
            }

            if self.data[0].as_bytes()[text_index] != *byte.borrow() { // bug
                return None;
            }

            node_byte_count += 1;
            pattern_len += 1;
        }

        Some(node.from(0) + node_byte_count - pattern_len) // bug
    }

    fn is_leaf(&self, node: &Node) -> bool {
        self.data[0].len() == node.to(0)
    }

    fn longest_repeat(&self, at: usize) -> &'a str {
        if at >= self.word_count {
            return "";
        }

        let mut stack = Stack::new();
        stack.push((&self.root, 0));
        let (mut ret_from, mut repeat_len) = (0, 0);
        while let Some((node, len)) = stack.pop() {
            if self.is_leaf(node) {
                let total_len = len - node.len(at);
                if total_len > repeat_len {
                    repeat_len = total_len;
                    ret_from = node.from(at) - repeat_len;
                }

                continue;
            }

            let children = node.nodes.iter().filter_map(|x| x.0.as_ref())
                .filter(|x| x.views[at].is_some());
            children.for_each(|x| stack.push((x, len + x.len(0))));
        }

        &self.data[at][ret_from .. ret_from + repeat_len]
    }

    fn longest_common_substring(&self) -> &'a str {
        let mut stack = Stack::new();
        stack.push((&self.root, 0));
        let (mut ret_from, mut repeat_len) = (0, 0);
        while let Some((node, len)) = stack.pop() {
            dbg!(node);
            if node.markmask != 0b11u8 {
                let total_len = len - node.len(0);
                if total_len > repeat_len {
                    repeat_len = total_len;
                    ret_from = node.from(0) - repeat_len;
                }

                continue;
            }

            let children = node.nodes.iter().filter_map(|x| x.0.as_ref());
            children.for_each(|x| stack.push((x, len + x.len(0))));
        }

        &self.data[0][ret_from .. ret_from + repeat_len]
    }
}

#[cfg(debug_assertions)]
impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("view", &self.views)
            .field("suffix_link", &self.suffix_link)
            .field("markmask", &format!("{:b}", self.markmask))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;
    const DEFAULT_MARKS: u8 = 1u8;

    fn clone_leaf(from: &Child) -> Child {
        Child(from.0.as_ref().map(|from| {
            Box::new(Node {
                views: from.views.clone(),
                nodes: make_children(),
                suffix_link: Link::default(),
                markmask: from.markmask,
            })
        }))
        
    }

    struct NodesBuild {
        nodes: [Child; 256],
    }

    impl NodesBuild {
        fn new() -> Self {
            Self { nodes: make_children() }
        }

        fn add(mut self, key: char, value: Node) -> Self {
            self.nodes[key as usize] = value.into();
            self
        }

        fn run(self) -> [Child; 256] {
            self.nodes
        }
    }

    impl PartialEq for Link {
        fn eq(&self, other: &Self) -> bool {
            if let (Some(left), Some(right)) = (self.0, other.0) {
                unsafe {
                    return dbg!(left.as_ref()) == dbg!(right.as_ref());
                }
            }

            (None, None) == (self.0, other.0)
        }
    }

    impl PartialEq for Node {
        fn eq(&self, other: &Self) -> bool {
            let res = self.views == other.views;
            res && self.suffix_link == other.suffix_link
        }
    }

    impl From<&Child> for Link {
        fn from(ch: &Child) -> Self {
            ch.0.as_ref().unwrap().as_ref().into()
        }
    }

    fn end_node(data: &str, expected_endptr: Rc<RefCell<usize>>) -> Node {
        Node {
            markmask: DEFAULT_MARKS,
            views: vec![Some((data.len(), data.len()))],
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

    fn select_ref<'b, 'a: 'b>(nodes: &'b [Child; 256], path: &[u8]) -> &'b Node {
        let ret = select(&nodes, path);
        unsafe { mem::transmute(ret) }
    }

    fn select<'b, 'a: 'b>(nodes: &'b [Child; 256], path: &[u8]) -> *const Node {
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
    ///      * abc$
    ///     /
    /// root -* ab$
    ///     \
    ///      * c$
    fn no_repeats() {
        let data = "abc";
        let tree = SuffixTree::new(data, END);
        let expected_endptr = Rc::new(RefCell::new(data.len()));
        let expected_tree = SuffixTree{
            data: vec![data],
            word_count: 1,
            end: END,
            root: Box::new(Node {
                markmask: DEFAULT_MARKS,
                views: vec![Some((0, ROOT_TO))],
                nodes: NodesBuild::new()
                    .add(END as char, end_node(data, expected_endptr.clone()))
                    .add('a', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((0, data.len()))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('b', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((1, data.len()))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('c', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((2, data.len()))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .run(),
                suffix_link: Link::default(),
            }),
        };

        test_nodes(&tree.root.nodes, &expected_tree.root.nodes);
    }

    #[test]
    #[ignore = "Test causes memory fault"]
    fn memcrash() {
        unsafe {
            TR = match Trace::new("/tmp/two_repeats") {
                Ok(x) => Some(x),
                Err(e) => panic!("OMG {}", e),
            };
        }

        let mut trace = unsafe {TR.as_mut().unwrap()};
        let mut current_end = 0usize;

        let input = "ababx";

        let mut root = Box::new(Node::new(0, ROOT_TO));
        root.markmask = 0b11;
        let mut active_point = ActivePoint::new(input, root, END, 1);

        {
            let key = b'a';
            let byte = &key;
            current_end += 1;
            active_point.edge = Some(*byte);
            let mut last_created_node = Link::default();
            let mut tr = trace.object(format!("`{}` `{}`", *byte as char, *byte).as_ref());
            let node: &Node = active_point.add_node(current_end);
            let inserted_node_link = node.into();
            last_created_node.set_suffix(inserted_node_link);
            last_created_node = inserted_node_link;
        }

        {
            current_end += 1;
            let mut last_created_node = Link::default();
            let key = b'b';
            let byte = &key;
            active_point.edge = Some(*byte);
            let mut tr = trace.object(format!("`{}` `{}`", *byte as char, *byte).as_ref());
            let node: &Node = active_point.add_node(current_end);
            let inserted_node_link = node.into();
            last_created_node.set_suffix(inserted_node_link);
            last_created_node = inserted_node_link;
        }

        {
            let mut last_created_node = Link::default();
            current_end += 1;
            active_point.length += 1;
        }

        {
            let mut last_created_node = Link::default();
            current_end += 1;
            active_point.length += 1;
        }

        {
            let mut last_created_node = Link::default();
            current_end += 1;
            let ref byte = b'a';
            active_point.edge = Some(*byte);
            let node = active_point.split_node(*byte, current_end);
            let inserted_node_link = node.into();
            last_created_node.set_suffix(inserted_node_link);
            last_created_node = inserted_node_link;

            active_point.length -= 1;
            active_point.edge = Some(b'a'); // It's not designed by Ukknen algorithm but allows to corrupt memory

            let ref byte = b'a';
            active_point.edge = Some(*byte);
            let node = active_point.split_node(*byte, current_end);
            let inserted_node_link: Link = node.into();
            last_created_node.set_suffix(inserted_node_link);
            last_created_node = inserted_node_link;
        }

        // split
        // active_point.node = inserted_node_link.0.unwrap();
        // active_point.length = 1;
        // active_point.split_node(b'a', b'x', current_end.clone());
        // end split

        // let key = b'c';
        // let byte = &key;
        // let mut tr = trace.object(format!("`{}` `{}`", *byte as char, *byte).as_ref());
        // let node: &Node = active_point.add_node(key, current_end.clone(), &mut tr);
        // let inserted_node_link = node.into();
        // last_created_node.set_suffix(inserted_node_link);
        // last_created_node = inserted_node_link;

        // let tree = SuffixTree::new(input);
    }

    #[test]
    /// Tests splitting a leaf node and linking between splitted nodes.
    ///
    /// assets/str/suffixtree/two_repeats.dot
    fn two_repeats() {
        let data = "abcabx";
        let tree = SuffixTree::new(data, END);
        let expected_endptr = Rc::new(RefCell::new(data.len()));
        let anode_nodes = NodesBuild::new()
            .add('c', Node{
                markmask: DEFAULT_MARKS,
                views: vec![Some((2, data.len()))],
                suffix_link: Link::default(),
                nodes: NodesBuild::new().run(),
            })
            .add('x', Node{
                markmask: DEFAULT_MARKS,
                views: vec![Some((5, data.len()))],
                suffix_link: Link::default(),
                nodes: NodesBuild::new().run(),
            })
            .run();

        let mut bnode_nodes = make_children();
        (0 .. 256).for_each(|i| bnode_nodes[i] = clone_leaf(&anode_nodes[i]));

        let mut expected_tree = SuffixTree{
            data: vec![data],
            word_count: 1,
            end: END,
            root: Box::new(Node {
                markmask: DEFAULT_MARKS,
                views: vec![Some((0, ROOT_TO))],
                nodes: NodesBuild::new()
                    .add(END as char, end_node(data, expected_endptr.clone()))
                    .add('a', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((0, 2))],
                        suffix_link: Link::default(),
                        nodes: anode_nodes,
                    })
                    .add('b', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((1, 2))],
                        suffix_link: Link::default(),
                        nodes: bnode_nodes,
                    })
                    .add('c', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((2, data.len()))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('x', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((5, data.len()))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .run(),
                suffix_link: Link::default(),
            }),
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

    // impl<'a> Child<'a> {
    //     fn suffix_link_ref(&self) -> &Node {
    //         unsafe {
    //             self.0.as_ref().unwrap().suffix_link.0.as_ref().unwrap().as_ref()
    //         }
    //     }
    // }

    #[test]
    /// Tests splitting a leaf node, linking between splitted nodes and following edge.
    /// See previous state of the tree in `two_repeats` docs.
    ///
    /// assets/str/suffixtree/three_repeats.dot
    fn three_repeats() {
        let data = "abcabxabcd";
        let tree = SuffixTree::new(data, END);
        let expected_endptr = Rc::new(RefCell::new(10));
        let cnode_nodes = NodesBuild::new()
            .add('a', Node{
                markmask: DEFAULT_MARKS,
                views: vec![Some((3, data.len()))],
                suffix_link: Link::default(),
                nodes: NodesBuild::new().run(),
            })
            .add('d', Node{
                markmask: DEFAULT_MARKS,
                views: vec![Some((9, data.len()))],
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
                markmask: DEFAULT_MARKS,
                views: vec![Some((2, 3))],
                suffix_link: Link::default(),
                nodes: b_cnode_nodes,
            })
            .add('x', Node{
                markmask: DEFAULT_MARKS,
                views: vec![Some((5, data.len()))],
                suffix_link: Link::default(),
                nodes: NodesBuild::new().run(),
            })
            .run();

        let anode_nodes = NodesBuild::new()
            .add('c', Node{
                markmask: DEFAULT_MARKS,
                views: vec![Some((2, 3))],
                suffix_link: Link::default(),
                nodes: ab_cnode_nodes,
            })
            .add('x', Node{
                markmask: DEFAULT_MARKS,
                views: vec![Some((5, data.len()))],
                suffix_link: Link::default(),
                nodes: NodesBuild::new().run(),
            })
            .run();

        let mut expected_tree = SuffixTree{
            data: vec![data],
            word_count: 1,
            end: END,
            root: Box::new(Node {
                markmask: DEFAULT_MARKS,
                views: vec![Some((0, ROOT_TO))],
                nodes: NodesBuild::new()
                    .add(END as char, end_node(data, expected_endptr.clone()))
                    .add('a', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((0, 2))],
                        suffix_link: Link::default(),
                        nodes: anode_nodes,
                    })
                    .add('b', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((1, 2))],
                        suffix_link: Link::default(),
                        nodes: bnode_nodes,
                    })
                    .add('c', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((2, 3))],
                        suffix_link: Link::default(),
                        nodes: cnode_nodes,
                    })
                    .add('d', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((9, data.len()))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('x', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((5, data.len()))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .run(),
                suffix_link: Link::default(),
            }),
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

    #[test]
    /// Tests adding a node from a splitted one.
    ///
    /// The `a` node of the root is splitted when `d` is processed. Then processing `a` the
    /// algorithm follows `a` node of the root (reducing active length). Then processing `k` it
    /// extends the `a` node.
    ///
    /// assets/str/suffixtree/inner_node_extend.dot
    fn inner_node_extend() {
        let data = "abcadak";
        let tree = SuffixTree::new(data, END);
        let expected_endptr = Rc::new(RefCell::new(data.len()));

        let mut expected_tree = SuffixTree{
            data: vec![data],
            word_count: 1,
            end: END,
            root: Box::new(Node {
                markmask: DEFAULT_MARKS,
                views: vec![Some((0, ROOT_TO))],
                nodes: NodesBuild::new()
                    .add(END as char, end_node(data, expected_endptr.clone()))
                    .add('a', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((0, 1))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new()
                            .add('k', Node{
                                markmask: DEFAULT_MARKS,
                                views: vec![Some((6, data.len()))],
                                nodes: make_children(),
                                suffix_link: Link::default(),
                            })
                            .add('d', Node{
                                markmask: DEFAULT_MARKS,
                                views: vec![Some((4, data.len()))],
                                nodes: make_children(),
                                suffix_link: Link::default(),
                            })
                            .add('b', Node{
                                markmask: DEFAULT_MARKS,
                                views: vec![Some((1, data.len()))],
                                nodes: make_children(),
                                suffix_link: Link::default(),
                            })
                            .run(),
                    })
                    .add('b', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((1, data.len()))],
                        suffix_link: Link::default(),
                        nodes: make_children(),
                    })
                    .add('c', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((2, data.len()))],
                        suffix_link: Link::default(),
                        nodes: make_children(),
                    })
                    .add('d', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((4, data.len()))],
                        suffix_link: Link::default(),
                        nodes: make_children(),
                    })
                    .add('k', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((6, data.len()))],
                        suffix_link: Link::default(),
                        nodes: make_children(),
                    })
                    .run(),
                suffix_link: Link::default(),
            }),
        };

        test_nodes(&tree.root.nodes, &expected_tree.root.nodes);
    }

    #[test]
    /// This test case seems redudant as it's covers the same as `undefined_repeat` test case.
    ///
    /// assets/str/suffixtree/pair_of_letters.dot
    fn pair_of_letters() {
        let data = "dd";
        let tree = SuffixTree::new(data, END);
        let expected_endptr = Rc::new(RefCell::new(data.len()));

        let mut expected_tree = SuffixTree{
            data: vec![data],
            word_count: 1,
            end: END,
            root: Box::new(Node {
                markmask: DEFAULT_MARKS,
                views: vec![Some((0, ROOT_TO))],
                nodes: NodesBuild::new()
                    .add(END as char, end_node(data, expected_endptr.clone()))
                    .add('d', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((0, 1))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new()
                            .add(END as char, end_node(data, expected_endptr.clone()))
                            .add('d', Node {
                                markmask: DEFAULT_MARKS,
                                views: vec![Some((1, data.len()))],
                                suffix_link: Link::default(),
                                nodes: make_children(),
                            }).run(),
                    }).run(),
                    suffix_link: Link::default(),
            }),
        };

        test_nodes(&tree.root.nodes, &expected_tree.root.nodes);
    }

    #[test]
    /// tests insertion of last unprocessed repeat.
    ///
    /// assets/str/suffixtree/undefined_repeat.dot
    fn undefined_repeat() {
        let data = "abab";
        let tree = SuffixTree::new(data, END);
        let expected_endptr = Rc::new(RefCell::new(data.len()));
        let mut expected_tree = SuffixTree{
            data: vec![data],
            word_count: 1,
            end: END,
            root: Box::new(Node {
                markmask: DEFAULT_MARKS,
                views: vec![Some((0, ROOT_TO))],
                nodes: NodesBuild::new()
                    .add(END as char, end_node(data, expected_endptr.clone()))
                    .add('a', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((0, 2))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new()
                            .add('a', Node{
                                markmask: DEFAULT_MARKS,
                                views: vec![Some((2, data.len()))],
                                suffix_link: Link::default(),
                                nodes: make_children(),
                            })
                            .add(END as char, Node{
                                markmask: DEFAULT_MARKS,
                                views: vec![Some((4, data.len()))],
                                suffix_link: Link::default(),
                                nodes: make_children(),
                            }).run(),
                    })
                    .add('b', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((1, 2))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new()
                            .add('a', Node{
                                markmask: DEFAULT_MARKS,
                                views: vec![Some((2, data.len()))],
                                suffix_link: Link::default(),
                                nodes: make_children(),
                            })
                            .add(END as char, Node{
                                markmask: DEFAULT_MARKS,
                                views: vec![Some((4, data.len()))],
                                suffix_link: Link::default(),
                                nodes: make_children(),
                            }).run(),
                    })
                    .run(),
                suffix_link: Link::default(),
            }),
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

    #[test]
    /// The test for a case when a splitted node has a suffix link to a splitted node which was
    /// created on the previous iteration of 'outer loop.
    ///
    /// This happens when `any` suffix is being inserted. It splits the `a` node of the root after
    /// `n` letter so the new branch carries only `y`.
    ///         * (xnyany)
    ///   (an) /
    ///       *-* (y)
    ///       |
    ///       |
    /// root -|
    ///
    /// As the node is splitted then we store it into last_created_node, decrement remainder as well
    /// active_point.length. Then we need to insert `ny`. As it was inserted when `ny` suffix was
    /// being processed then the `n` node has `x` and `y` nodes. It looks so:
    ///         * (xnyany)
    ///   (an) /
    ///       *-* (y)
    ///       |
    ///       |     *(xnyany)
    ///       |(n) /
    /// root -----*-*(yany)
    ///
    /// So it means that we can't insert `ny` suffix because it's already there the only thing to
    /// do is following the `n` node of the root, decrement active_point.length. Now staying in the
    /// `n` node we have to insert only `y` but it's there too so we just increment active_point
    /// but also we link the `an` node of the root to the `n` node of the root. (orange line in the
    /// graphviz graph)
    ///
    /// This trick works only once as we store last linked node (last_created_node) until the next
    /// iteration of 'outer loop which starts right after the incrementing of the
    /// active_point.lengh.
    ///
    /// If don't create the link on this step then we will lose `nz` (root -> n -> z) suffix while
    /// inserting `anz` because the `an` will not have the suffix link. This particular input
    /// simply crashes if don't do this thing.
    ///
    /// assets/str/suffixtree/post_suffix_linking.dot
    fn post_suffix_linking() {
        let tree = SuffixTree::new("anxnyanyanz", END);
    }


    #[test]
    /// This test case ensures that pointer to the last created node is still relevant after
    /// splitting an inner node which the last created node is child of.
    ///
    /// assets/str/suffixtree/suffix_link_from_recreated_node.dot
    /// The red node changes its parent on the second step but it must keep the same place in RAM
    /// to keep last_created_node valid.
    fn suffix_link_from_recreated_node() {
        let tree = SuffixTree::new("GAAA", END);
    }

    #[test]
    /// Test splitting inner node (if a node is inner then it's splitted already)
    ///
    /// Render assets/str/suffixtree/inner_node_split.dot to see details. The last step isn't drawn.
    /// It has only a new edge from root at key `$`.
    fn inner_node_split() {
        let data = "banana";
        let tree = SuffixTree::new(data, END);
        let expected_endptr = Rc::new(RefCell::new(data.len()));
        let mut expected_tree = SuffixTree{
            data: vec![data],
            word_count: 1,
            end: END,
            root: Box::new(Node {
                markmask: DEFAULT_MARKS,
                views: vec![Some((0, ROOT_TO))],
                nodes: NodesBuild::new()
                    .add(END as char, end_node(data, expected_endptr.clone()))
                    .add('a', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((1, 2))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new()
                            .add('n', Node{
                                markmask: DEFAULT_MARKS,
                                views: vec![Some((2, 4))],
                                suffix_link: Link::default(),
                                nodes: NodesBuild::new()
                                    .add('n', Node {
                                        markmask: DEFAULT_MARKS,
                                        views: vec![Some((4, data.len()))],
                                        suffix_link: Link::default(),
                                        nodes: make_children(),
                                    })
                                    .add(END as char, end_node(data, expected_endptr.clone()))
                                    .run(),
                            })
                            .add(END as char, end_node(data, expected_endptr.clone()))
                            .run(),
                    })
                    .add('b', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((0, data.len()))],
                        suffix_link: Link::default(),
                        nodes: make_children(),
                    })
                    .add('n', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((2, 4))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new()
                            .add('n', Node{
                                markmask: DEFAULT_MARKS,
                                views: vec![Some((4, data.len()))],
                                suffix_link: Link::default(),
                                nodes: make_children(),
                            })
                            .add(END as char, end_node(data, expected_endptr.clone()))
                            .run(),
                    })
                    .run(),
                suffix_link: Link::default(),
            }),
        };

        link_inner_nodes(&mut expected_tree.root.nodes, &[b'a', b'n'], &[b'n']);
        link_inner_nodes(&mut expected_tree.root.nodes, &[b'n'], &[b'a']);
        test_nodes(&tree.root.nodes, &expected_tree.root.nodes);
        test_nodes(
            &select_ref(&tree.root.nodes, &[b'a']).nodes,
            &select_ref(&expected_tree.root.nodes, &[b'a']).nodes,
        );

        test_nodes(
            &select_ref(&tree.root.nodes, &[b'n']).nodes,
            &select_ref(&expected_tree.root.nodes, &[b'n']).nodes,
        );

        test_nodes(
            &select_ref(&tree.root.nodes, &[b'a', b'n']).nodes,
            &select_ref(&expected_tree.root.nodes, &[b'a', b'n']).nodes,
        );

        test_nodes(
            &select_ref(&tree.root.nodes, &[b'n', b'n']).nodes,
            &select_ref(&expected_tree.root.nodes, &[b'n', b'n']).nodes,
        );

    }

    mod find {
        use super::*;

        #[test]
        fn positive(){
            let tree = SuffixTree::new("abcabx", END);
            let position = tree.find("bx".bytes());
            assert_eq!(position, Some(4));

            let tree = SuffixTree::new("a", END);
            let position = tree.find("a".bytes());
            assert_eq!(position, Some(0));

            let tree = SuffixTree::new(LOREM_IPSUM, END);
            let position = tree.find("dummy".bytes());
            assert_eq!(position, Some(23));
        }

        #[test]
        fn negative(){
            let tree = SuffixTree::new(LOREM_IPSUM, END);
            let position = tree.find("shadow".bytes());
            assert_eq!(position, None);

            let tree = SuffixTree::new("", END);
            let position = tree.find("some".bytes());
            assert_eq!(position, None);

            let tree = SuffixTree::new("abc", END);
            let position = tree.find("ac".bytes());
            assert_eq!(position, None);
        }
    }

    mod longest_repeated_overlapped_substring {
        use super::*;

        #[test]
        fn basic() {
            let tree = SuffixTree::new("anxnyanyanz", END);
            assert_eq!(tree.longest_repeat(0), "nyan");

            let tree = SuffixTree::new("GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", END);
            assert_eq!(tree.longest_repeat(0), "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");

            let tree = SuffixTree::new("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAG", END);
            assert_eq!(tree.longest_repeat(0), "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");

            let tree = SuffixTree::new("AAAAAAAAAAAAAAAAAAGAAAAAAAAAAAAAAAAAA", END);
            assert_eq!(tree.longest_repeat(0), "AAAAAAAAAAAAAAAAAA");

            let tree = SuffixTree::new(LOREM_IPSUM, END);
            assert_eq!(tree.longest_repeat(0), " Lorem Ipsum ");
        }

        #[test]
        fn absent_word() {
            let tree = SuffixTree::new(LOREM_IPSUM, END);
            assert_eq!(tree.longest_repeat(1), "");
        }

    }

    #[ignore]
    #[test]
    fn longest_common_substring() {
        let left = "qwa";
        let right = "qwe";
        let total = format!("{}{}{}", left, DELIMETER, right);// bug
        let tree = SuffixTree::new(&total, END);
        assert_eq!(tree.longest_common_substring(), "qw");
    }

    #[test]
    fn experiment() {
        assert_eq!(b'1', 0x31);
        assert_ne!(b'1', 0x01);
    }
}
