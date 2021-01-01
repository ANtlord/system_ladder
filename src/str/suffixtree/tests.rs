use super::*;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

const DEFAULT_MARKS: u8 = 1u8;
const END: u8 = b'\0';
const DELIMETER: u8 = 0x01; // SOH like in FIX protocol.

static LOREM_IPSUM: &str = include_str!("../lorem_ipsum.txt");

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

fn end_node(data: &str) -> Node {
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
                .add(END as char, end_node(data))
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
    let mut current_end = 0usize;
    let input = "ababx";

    let mut root = Box::new(Node::new(0, 0, ROOT_TO));
    root.markmask = 0b11;
    let mut active_point = ActivePoint::new(input, root, END, 1);

    {
        let key = b'a';
        let byte = &key;
        current_end += 1;
        active_point.edge = *byte;
        let mut last_created_node = Link::default();
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
        active_point.edge = *byte;
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
        active_point.edge = *byte;
        let node = active_point.split_node(*byte, current_end);
        let inserted_node_link = node.into();
        last_created_node.set_suffix(inserted_node_link);
        last_created_node = inserted_node_link;

        active_point.length -= 1;
        active_point.edge = b'a'; // It's not designed by Ukknen algorithm but allows to corrupt memory

        let ref byte = b'a';
        active_point.edge = *byte;
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
                .add(END as char, end_node(data))
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
                .add(END as char, end_node(data))
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
                .add(END as char, end_node(data))
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
                .add(END as char, end_node(data))
                .add('d', Node{
                    markmask: DEFAULT_MARKS,
                    views: vec![Some((0, 1))],
                    suffix_link: Link::default(),
                    nodes: NodesBuild::new()
                        .add(END as char, end_node(data))
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
                .add(END as char, end_node(data))
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
                .add(END as char, end_node(data))
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
                            .add(END as char, end_node(data))
                                .run(),
                        })
                    .add(END as char, end_node(data))
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
                .add(END as char, end_node(data))
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

mod two_words {
    use super::*;

    #[test]
    fn try_follow_edge_simple() {
        let data = "abc";
        let tree = SuffixTree::new(data, END);
        let tree = tree.extend("bc");
    }

    #[test]
    fn try_follow_edge() {
        let data = "dd";
        let tree = SuffixTree::new(data, END);
        println!("======================================================================> first success");
        let tree = tree.extend(data);
        let expected_endptr = Rc::new(RefCell::new(data.len()));
        let expected_endptr = Rc::new(RefCell::new(data.len()));

        let mut expected_tree = SuffixTree{
            data: vec![data],
            word_count: 1,
            end: END,
            root: Box::new(Node {
                markmask: DEFAULT_MARKS,
                views: vec![Some((0, ROOT_TO))],
                nodes: NodesBuild::new()
                    .add(END as char, Node {
                        markmask: DEFAULT_MARKS,
                        views: vec![None, Some((data.len(), data.len()))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('d', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((0, 1)), Some((0, 1))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new()
                            .add(END as char, end_node(data))
                            .add('d', Node {
                                markmask: DEFAULT_MARKS,
                                views: vec![Some((1, data.len())), Some((1, data.len()))],
                                suffix_link: Link::default(),
                                nodes: make_children(),
                            }).run(),
                    }).run(),
                    suffix_link: Link::default(),
            }),
        };

        test_nodes(&tree.root.nodes, &expected_tree.root.nodes);
        test_nodes(
            &select_ref(&tree.root.nodes, &[b'd']).nodes,
            &select_ref(&expected_tree.root.nodes, &[b'd']).nodes,
        );
    }

    #[ignore]
    #[test]
    fn split_node() {
        let data = "abc";
        let tree = SuffixTree::new(data, END);
        println!("======================================================================> first success");
        let tree = tree.extend("abd");

        let expected_endptr = Rc::new(RefCell::new(data.len()));
        let expected_tree = SuffixTree{
            data: vec![data],
            word_count: 1,
            end: END,
            root: Box::new(Node {
                markmask: DEFAULT_MARKS,
                views: vec![Some((0, ROOT_TO))],
                nodes: NodesBuild::new()
                    .add(END as char, Node {
                        markmask: DEFAULT_MARKS,
                        views: vec![None, Some((data.len(), data.len()))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('a', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((0, 2)), Some((0, 2))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('b', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((1, 2)), Some((1, 2))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('c', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![Some((2, data.len())), None],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .add('d', Node{
                        markmask: DEFAULT_MARKS,
                        views: vec![None, Some((2, data.len()))],
                        suffix_link: Link::default(),
                        nodes: NodesBuild::new().run(),
                    })
                    .run(),
                suffix_link: Link::default(),
            }),
        };

        test_nodes(&tree.root.nodes, &expected_tree.root.nodes);
    }
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

use std::mem::size_of_val;

#[test]
fn experiment() {
    // let data = "aba";
    // let tree = SuffixTree::new(data, END);
    // let expected_endptr = Rc::new(RefCell::new(data.len()));
}
