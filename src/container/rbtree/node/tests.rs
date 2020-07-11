use super::*;

type Nodeu8 = NonNull<Node<u8>>;
type LongBranch = (Nodeu8, Nodeu8, Nodeu8, Nodeu8);
type Branch = (Nodeu8, Nodeu8, Nodeu8);

unsafe fn make_long_branch(a: u8, g: u8, p: u8, l: u8) -> LongBranch {
    let mut ancestor = Node::head(to_heap(a));
    let mut grandparent = ancestor.as_mut().add(to_heap(g));
    let mut parent = grandparent.as_mut().add(to_heap(p));
    let mut leaf = parent.as_mut().add(to_heap(l));
    (ancestor, grandparent, parent, leaf)
}

unsafe fn make_branch(h: u8, c: u8, g: u8) -> Branch {
    let mut head = Node::head(to_heap(h));
    let mut child = head.as_mut().add(to_heap(c));
    let mut grandchild = child.as_mut().add(to_heap(g));
    (head, child, grandchild)
}

#[test]
fn test_cmp() {
    unsafe {
        let mut head = Node::head(to_heap(2));
        let right = Node::head(to_heap(1));
        let left_child = head.as_mut().add(to_heap(1));
        assert_eq!(head, head);
        assert_ne!(head, right);
        assert_eq!(left_child.as_ref().parent.unwrap(), head);
        assert_eq!(head.as_ref().left.unwrap(), left_child);
        assert_eq!(Some(head.as_ref().left.unwrap()), Some(left_child));
        assert_eq!(
            head.as_ref().left.unwrap().as_ref() as *const _,
            left_child.as_ref() as *const _
        );
    }
}

#[test]
fn sibling() {
    unsafe {
        let mut head = Node::head(to_heap(2));
        let left = head.as_mut().add(to_heap(1));
        let right = head.as_mut().add(to_heap(3));
        assert_eq!(head.as_ref().left.unwrap(), left);
        assert_eq!(head.as_ref().right.unwrap(), right);
        assert_eq!(left.as_ref().sibling().unwrap(), right);
        assert_eq!(right.as_ref().sibling().unwrap(), left);
    }
}

#[test]
fn grandparent() {
    unsafe {
        let (head, child, grandchild) = make_branch(2, 3, 4);
        assert_eq!(grandchild.as_ref().grandparent().unwrap(), head);
    }
}

#[test]
fn uncle() {
    unsafe {
        let (mut head, child1, grandchild) = make_branch(2, 3, 4);
        let mut child2 = head.as_mut().add(to_heap(1));
        assert_eq!(grandchild.as_ref().uncle().unwrap(), child2);
    }
}

#[test]
fn add() {
    unsafe {
        let (mut head, child1, grandchild) = make_branch(2, 3, 4);
        let mut child2 = head.as_mut().add(to_heap(1));
        assert_eq!(child1.as_ref().parent.unwrap(), head);
        assert_eq!(child2.as_ref().parent.unwrap(), head);
        assert_eq!(grandchild.as_ref().parent.unwrap(), child1);
        assert_eq!(head.as_ref().left.unwrap(), child2);
        assert_eq!(head.as_ref().right.unwrap(), child1);
        assert_eq!(child1.as_ref().right.unwrap(), grandchild);
    }
}

#[test]
fn rotate_left_no_ancestor() {
    unsafe {
        let (mut grandparent, parent, leaf) = make_branch(2, 3, 4);
        grandparent.as_mut().rotate_left().unwrap();
        assert_eq!(leaf.as_ref().sibling().unwrap(), grandparent);
        assert_eq!(leaf.as_ref().parent.unwrap(), parent);
        assert_eq!(grandparent.as_ref().parent.unwrap(), parent);
        assert_eq!(parent.as_ref().left.unwrap(), grandparent);
        assert_eq!(parent.as_ref().right.unwrap(), leaf);
    }
}

#[test]
fn rotate_right_left_no_ancestor() {
    unsafe {
        //  2
        //   \
        //    4
        //   /
        //  3
        let (mut grandparent, mut parent, leaf) = make_branch(2, 4, 3);
        parent.as_mut().rotate_right().unwrap();
        //  2
        //   \
        //    3
        //     \
        //      4
        assert_eq!(leaf.as_ref().parent.unwrap(), grandparent);
        assert_eq!(parent.as_ref().parent.unwrap(), leaf);
        assert_eq!(grandparent.as_ref().right.unwrap(), leaf);
        grandparent.as_mut().rotate_left().unwrap();
        //    3
        //   / \
        //  2   4
        assert_eq!(leaf.as_ref().parent, None);
        assert_eq!(parent.as_ref().parent.unwrap(), leaf);
        assert_eq!(grandparent.as_ref().parent.unwrap(), leaf);
    }
}

#[test]
fn rotate_right_no_ancestor() {
    unsafe {
        let (mut grandparent, parent, leaf) = make_branch(4, 3, 2);
        grandparent.as_mut().rotate_right().unwrap();
        assert_eq!(leaf.as_ref().sibling().unwrap(), grandparent);
        assert_eq!(grandparent.as_ref().parent.unwrap(), parent);
        assert_eq!(parent.as_ref().right.unwrap(), grandparent);
        assert_eq!(parent.as_ref().left.unwrap(), leaf);
    }
}

#[test]
fn rotate_left_with_ancestor() {
    unsafe {
        let (ancestor, mut grandparent, parent, leaf) = make_long_branch(2, 3, 4, 5);
        grandparent.as_mut().rotate_left().unwrap();
        assert_eq!(leaf.as_ref().sibling().unwrap(), grandparent);
        assert_eq!(grandparent.as_ref().parent.unwrap(), parent);
        assert_eq!(parent.as_ref().left.unwrap(), grandparent);
        assert_eq!(parent.as_ref().right.unwrap(), leaf);
        assert_eq!(parent.as_ref().parent.unwrap(), ancestor);
        assert_eq!(ancestor.as_ref().right.unwrap(), parent);
    }
}

#[test]
fn rotate_right_with_ancestor() {
    unsafe {
        let (ancestor, mut grandparent, parent, leaf) = make_long_branch(5, 4, 3, 2);
        grandparent.as_mut().rotate_right().unwrap();
        assert_eq!(leaf.as_ref().sibling().unwrap(), grandparent);
        assert_eq!(grandparent.as_ref().parent.unwrap(), parent);
        assert_eq!(parent.as_ref().right.unwrap(), grandparent);
        assert_eq!(parent.as_ref().left.unwrap(), leaf);
        assert_eq!(parent.as_ref().parent.unwrap(), ancestor);
        assert_eq!(ancestor.as_ref().left.unwrap(), parent);
    }
}

#[test]
fn repair_left_left() {
    //     a(b)
    //    /   \
    //   p(r)  n(b)
    //  /
    // n(r)
    //-------------
    //     p(r)
    //    /   \
    //   n(r)  a(b)
    //          \
    //           u(b)
    //---------------
    //     p(b)
    //    /   \
    //   n(r)  a(r)
    //          \
    //           u(b)
    unsafe {
        let (mut head, parent, node) = make_branch(8, 7, 6);
        let mut uncle = head.as_mut().add(to_heap(9));
        uncle.as_mut().color = Color::Black;
        let res = repair_step(node);
        check_node(node, Color::Red, None, None, parent, "node");
        check_node(head, Color::Red, None, uncle, parent, "head");
        check_node(parent, Color::Black, node, head, None, "parent");
        check_node(uncle, Color::Black, None, None, head, "uncle");
    }
}

#[test]
fn repair_left_right() {
    //     a(b)
    //    / \
    // p(r)  u(b)
    //    \
    //   n(r)
    // -----------
    //     a(b)
    //    /   \
    //   n(r)  u(b)
    //  /
    // p(r)
    // -----------
    //      n(b)
    //     / \
    //  p(r)  a(r)
    //         \
    //          u(b)
    unsafe {
        let (mut head, parent, node) = make_branch(8, 6, 7);
        let mut uncle = head.as_mut().add(to_heap(9));
        uncle.as_mut().color = Color::Black;
        let res = repair_step(node);
        check_node(node, Color::Black, parent, head, None, "node");
        check_node(head, Color::Red, None, uncle, node, "head");
        check_node(parent, Color::Red, None, None, node, "parent");
        check_node(uncle, Color::Black, None, None, head, "uncle");
    }
}

#[test]
fn repair_right_right() {
    //     a(b)
    //    / \
    // u(b)  p(r)
    //        \
    //         n(r)
    // -----------
    //     p(r)
    //    /   \
    //   a(b)  n(r)
    //  /
    // u(b)
    // -----------
    //     p(b)
    //    /   \
    //   a(r)  n(r)
    //  /
    // u(b)
    unsafe {
        let (mut head, parent, node) = make_branch(5, 7, 8);
        let mut uncle = head.as_mut().add(to_heap(4));
        uncle.as_mut().color = Color::Black;
        let res = repair_step(node);
        check_node(node, Color::Red, None, None, parent, "node");
        check_node(head, Color::Red, uncle, None, parent, "head");
        check_node(parent, Color::Black, head, node, None, "parent");
        check_node(uncle, Color::Black, None, None, head, "uncle");
    }
}

#[test]
fn repair_right_left() {
    //     a(b)
    //    / \
    // u(b)  p(r)
    //      /
    //     n(r)
    // -----------
    //     a(b)
    //    / \
    // u(b)  n(r)
    //        \
    //         p(b)
    // -----------
    //        n(b)
    //       / \
    //    a(r)  p(r)
    //     /
    //   u(b)
    unsafe {
        let (mut head, parent, node) = make_branch(5, 7, 6);
        let mut uncle = head.as_mut().add(to_heap(4));
        uncle.as_mut().color = Color::Black;
        let res = repair_step(node);
        check_node(node, Color::Black, head, parent, None, "node");
        check_node(head, Color::Red, uncle, None, node, "head");
        check_node(parent, Color::Red, None, None, node, "parent");
        check_node(uncle, Color::Black, None, None, head, "uncle");
    }
}

#[test]
fn repair_red_uncle() {
    //     a(b)
    //    / \
    // u(r)  p(r)
    //      /
    //     n(r)
    //-------------
    //     a(r)
    //    / \
    // u(b)  p(b)
    //      /
    //     n(r)
    unsafe {
        let (mut head, parent, node) = make_branch(5, 7, 6);
        let mut uncle = head.as_mut().add(to_heap(4));
        let next = repair_step(node);
        assert_eq!(next.unwrap(), head);
        check_node(head, Color::Red, uncle, parent, None, "head");
        check_node(uncle, Color::Black, None, None, head, "uncle");
        check_node(parent, Color::Black, node, None, head, "parent");
        check_node(node, Color::Red, None, None, parent, "node");
    }
}

#[test]
fn repair_straight_branch() {
    unsafe {
        // a(b)
        //  \
        //   p(r)
        //    \
        //     n(r)
        //---------
        //     p(b)
        //    / \
        // a(r)  n(r)
        let (head, parent, node) = make_branch(5, 6, 7);
        let res = repair_step(node);
        assert_eq!(res, None);
        check_node(node, Color::Red, None, None, parent, "node");
        check_node(head, Color::Red, None, None, parent, "head");
        check_node(parent, Color::Black, head, node, None, "parent");
    }
}

fn check_node<T>(
    node: NonNull<Node<T>>,
    color: Color,
    left: impl Into<Option<NonNull<Node<T>>>>,
    right: impl Into<Option<NonNull<Node<T>>>>,
    parent: impl Into<Option<NonNull<Node<T>>>>,
    message: impl Into<Option<&'static str>>,
) {
    let id = message.into().unwrap_or("");
    let msg = format!("{} has wrong", id);
    unsafe {
        assert_eq!(node.as_ref().left, left.into(), "{} left", msg);
        assert_eq!(node.as_ref().right, right.into(), "{} right", msg);
        assert_eq!(node.as_ref().parent, parent.into(), "{} parent", msg);
        match color {
            Color::Black => assert!(node.as_ref().color.is_black(), "{} color", msg),
            Color::Red => assert!(node.as_ref().color.is_red(), "{} color", msg),
        }
    }
}

#[test]
fn replace_child() {
    unsafe {
        let (mut head, parent, node) = make_branch(5, 6, 7);
        assert_eq!(head.as_ref().right.unwrap(), parent);
        assert_eq!(parent.as_ref().parent.unwrap(), head);
        head.as_mut().replace_child(node, parent);
        assert_eq!(head.as_ref().right.unwrap(), node);
    }
}
