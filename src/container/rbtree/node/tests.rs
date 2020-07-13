use super::*;
use std::mem;

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

#[test]
fn del_step_leaf() {
    unsafe {
        let (head, parent, mut node) = make_branch(5, 6, 7);
        let res = node.as_mut().del_step();
        assert_eq!(res, None);
        assert_eq!(parent.as_ref().right, None);
        assert_eq!(parent.as_ref().left, None);
    }
}

#[test]
fn del_step_head_with_replacement() {
    // it tests case when it removes head with a child.
    // Note: It's impossible to have a head with a grandchild. It inserts a grandchild, child
    // rotates (look repair_straight_branch test case).
    unsafe {
        let mut head = Node::head(to_heap(2));
        head.as_mut().add(to_heap(3));
        let res = head.as_mut().del_step();
        assert_eq!(res, None);
        assert_eq!(head.as_ref().value.as_ref(), &3);
        assert_eq!(head.as_ref().left, None);
        assert_eq!(head.as_ref().right, None);
    }
}

#[test]
fn del_step_internal_node_with_replacement() {
    // delete 2
    // a(1)     | a(1)
    //  \       |  \
    //   p(2)   |   n(3)
    //    \     |
    //     n(3) |
    unsafe {
        let (head, mut child, grandchild) = make_branch(1, 2, 3);
        child.as_mut().del_step();
        assert_eq!(head.as_ref().right.unwrap(), grandchild);
        assert_eq!(grandchild.as_ref().parent.unwrap(), head);
    }
}

#[test]
fn del_step_head_with_replacement_and_two_children() {
    // delete 2
    //   h(2)      |  h(3)
    //  /    \     |  /
    // l(1)   r(3) | l(1)
    unsafe {
        let head_val = to_heap(2);
        let right_val = to_heap(3);
        let left_val = to_heap(1);
        let mut head = Node::head(head_val);
        let right = head.as_mut().add(right_val);
        let left = head.as_mut().add(left_val);
        let res = head.as_mut().del_step();
        assert_eq!(res, Some(right));
        assert_eq!(head.as_ref().value, right_val);

        let res = res.unwrap().as_mut().del_step();
        assert_eq!(res, None);
        assert_eq!(head.as_ref().right, None);
        assert_eq!(head.as_ref().left.unwrap().as_ref().value, left_val);
    }
}

unsafe fn add_node<T: Ord>(mut parent: NonNull<Node<T>>, value: T) -> NonNull<Node<T>>  {
    parent.as_mut().add(to_heap(value))
}

#[test]
fn min_node() {
    unsafe {
        let (head, parent, node) = make_branch(5, 6, 7);
        {
            let min = head.as_ref().min_node();
            assert_eq!(min, None, "min must be None. head doesn't have the left child.")
        }

        {
            let left = add_node(head, 1);
            let min = head.as_ref().min_node();
            assert_eq!(min.unwrap(), left, "min node is not 1");
        }
    }
}

fn nodes_from_iter<T: Ord>(head_val: T, source: Vec<T>) -> NonNull<Node<T>> {
    let mut head = Node::head(to_heap(head_val));
    source.into_iter().for_each(|x| unsafe {
        add_node(head, x);
    });
    head
}

mod find_replacement {
    use super::*;

    #[test]
    fn left() {
        unsafe {
            let (head, parent, node) = make_branch(5, 6, 7);
            let replacement = head.as_ref().find_replacement();
            assert_eq!(replacement.unwrap(), parent);
        }
    }

    #[test]
    fn right() {
        unsafe {
            let mut head = Node::head(to_heap(2));
            let right = add_node(head, 3);
            let left = add_node(head, 1);
            let replacement = head.as_ref().find_replacement();
            assert_eq!(replacement.unwrap(), right);
        }
    }

    #[test]
    fn right_left() {
        unsafe {
            let right_left_val = 8;
            let head = nodes_from_iter(5, vec![1, 10, right_left_val]);
            let replacement = head.as_ref().find_replacement();
            assert_eq!(replacement.unwrap().as_ref().value.as_ref(), &right_left_val);
        }
    }
}

mod fix_double_black {
    use super::*;

    #[test]
    fn no_sibling() {
        unsafe {
            let head = nodes_from_iter(5, vec![1]);
            let res = head.as_ref().left.unwrap().as_mut().fix_double_black_step();
            assert_eq!(res, Some(head));
        }
    }

    #[test]
    fn red_right_sibling() {
        //   p(?)    |    s(b)
        //  /    \   |   /
        // n(b)  s(r)|  p(r)
        //           | /
        //           |n(b)
        unsafe {
            let parent = nodes_from_iter(5, vec![1, 10]);
            let mut sibling = parent.as_ref().right.unwrap();
            sibling.as_mut().color = Color::Red;
            let mut node = parent.as_ref().left.unwrap();
            node.as_mut().color = Color::Black;
            let res = node.as_mut().fix_double_black_step();
            assert_eq!(res, Some(node));
            check_node(node, Color::Black, None, None, parent, "leaf node");
            check_node(parent, Color::Red, node, None, sibling, "parent node");
            check_node(sibling, Color::Black, parent, None, None, "sibling node");
        }
    }

    #[test]
    fn red_left_sibling() {
        //   p(?)    | s(b)
        //  /    \   |  \
        // s(r)  n(b)|   p(r)
        //           |    \
        //           |     n(b)
        unsafe {
            let parent = nodes_from_iter(5, vec![1, 10]);
            let mut sibling = parent.as_ref().left.unwrap();
            sibling.as_mut().color = Color::Red;
            let mut right = parent.as_ref().right.unwrap();
            right.as_mut().color = Color::Black;
            let res = right.as_mut().fix_double_black_step();
            assert_eq!(res, Some(right));
            check_node(right, Color::Black, None, None, parent, "leaf node");
            check_node(parent, Color::Red, None, right, sibling, "parent node");
            check_node(sibling, Color::Black, None, parent, None, "sibling node");
        }
    }

    #[test]
    fn black_sibling_red_left_child() {
        //      p(?)    |    s(?)
        //     /    \   |   /    \
        //    s(b)  n(b)| sl(b)  p(b)
        //   /          |            \
        // sl(r)        |            n(b)
        unsafe {
            let parent = nodes_from_iter(5, vec![1, 10, 0]);
            let parent_color = parent.as_ref().color;
            let mut sibling = parent.as_ref().left.unwrap();
            sibling.as_mut().color = Color::Black;
            let mut sibling_left = sibling.as_ref().left.unwrap();
            sibling_left.as_mut().color = Color::Red;
            let mut node = parent.as_ref().right.unwrap();
            node.as_mut().color = Color::Black;
            assert_eq!(node.as_mut().fix_double_black_step(), None);
            check_node(parent, Color::Black, None, node, sibling, "parent node");
            check_node(sibling, parent_color, sibling_left, parent, None, "sibling node");
            check_node(sibling_left, Color::Black, None, None, sibling, "sibling_left node");
            check_node(node, Color::Black, None, None, parent, "leaf node");
        }
    }

    #[test]
    fn black_sibling_red_right_child() {
        //      p(?)    |   sr(?)
        //     /    \   |  /    \
        //    s(b)  n(b)| s(b)   p(b)
        //     \        |         \
        //      sr(r)   |          n(b)
        unsafe {
            let parent = nodes_from_iter(5, vec![1, 10, 2]);
            let parent_color = parent.as_ref().color;
            let mut sibling = parent.as_ref().left.unwrap();
            sibling.as_mut().color = Color::Black;
            let mut sibling_right = sibling.as_ref().right.unwrap();
            sibling_right.as_mut().color = Color::Red;
            let mut node = parent.as_ref().right.unwrap();
            node.as_mut().color = Color::Black;
            assert_eq!(node.as_mut().fix_double_black_step(), None);
            check_node(parent, Color::Black, None, node, sibling_right, "parent node");
            check_node(sibling, Color::Black, None, None, sibling_right, "sibling node");
            check_node(sibling_right, parent_color, sibling, parent, None, "sibling_right node");
            check_node(node, Color::Black, None, None, parent, "leaf node");
        }
    }

    #[test]
    fn black_sibling_without_red_child_and_black_parent() {
        //      p(b)    |   p(b)
        //     /    \   |  /    \
        //    s(b)  n(b)| s(r)   n(b)
        unsafe {
            let mut parent = nodes_from_iter(5, vec![1, 10]);
            parent.as_mut().color = Color::Black;
            let mut sibling = parent.as_mut().left.unwrap();
            sibling.as_mut().color = Color::Black;
            let res = parent.as_ref().right.unwrap().as_mut().fix_double_black_step();
            assert_eq!(res, Some(parent));
            assert!(parent.as_ref().left.unwrap().as_ref().color.is_red());
        }
    }

    fn black_sibling_without_red_child_and_red_parent() {
        //      p(r)    |   p(b)
        //     /    \   |  /    \
        //    s(b)  n(b)| s(b)   n(b)
        unsafe {
            let mut parent = nodes_from_iter(5, vec![1, 10]);
            parent.as_mut().color = Color::Red;
            let mut sibling = parent.as_mut().left.unwrap();
            sibling.as_mut().color = Color::Black;
            let res = parent.as_ref().right.unwrap().as_mut().fix_double_black_step();
            assert_eq!(res, None);
            assert!(parent.as_ref().color.is_black());
        }
    }
}
