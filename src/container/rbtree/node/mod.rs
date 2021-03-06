use std::cmp::Ord;
use std::collections::BTreeMap;
use std::fmt;
use std::mem;
use std::ptr::NonNull;

#[cfg(test)]
mod tests;

pub type NodePtr<T, P> = Option<NonNull<Node<T, P>>>;

fn zero_node_ptr<T, P>() -> NodePtr<T, P> {
    None
}

unsafe fn node_ptr<T, P>(reference: &mut Node<T, P>) -> NodePtr<T, P> {
    Some(NonNull::new_unchecked(reference))
}

#[derive(Clone, Copy)]
pub enum Color {
    Red,
    Black,
}

impl fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::Red => f.write_str("(r)"),
            Color::Black => f.write_str("(b)"),
        }
    }
}

impl Color {
    pub fn is_red(&self) -> bool {
        match self {
            Color::Black => false,
            Color::Red => true,
        }
    }

    pub fn is_black(&self) -> bool {
        match self {
            Color::Black => true,
            Color::Red => false,
        }
    }
}

pub struct Node<T, P> {
    pub parent: NodePtr<T, P>,
    pub left: NodePtr<T, P>,
    pub right: NodePtr<T, P>,
    pub color: Color,
    pub value: NonNull<T>,
    pub payload: P,
}

fn to_heap<T>(val: T) -> NonNull<T> {
    unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(val))) }
}

enum State<T, P> {
    Continue(NonNull<Node<T, P>>),
    Stop(NonNull<Node<T, P>>),
}

impl<T: Ord, P> Node<T, P> {
    pub unsafe fn add(&mut self, val: NonNull<T>, payload: P) -> NonNull<Self> {
        let mut ret = NonNull::new_unchecked(self);
        let mut node_opt = Some(ret);
        while let Some(mut node) = node_opt.take() {
            let mut child_node = if val.as_ref() < node.as_ref().value.as_ref() {
                if node.as_mut().left.is_none() {
                    let new = Self::red(val, node, payload);
                    node.as_mut().left.replace(new);
                    ret = new;
                    break;
                }

                node.as_mut().left
            } else if val.as_ref() > node.as_ref().value.as_ref() {
                if node.as_mut().right.is_none() {
                    let new = Self::red(val, node, payload);
                    node.as_mut().right.replace(new);
                    ret = new;
                    break;
                }

                node.as_mut().right
            } else {
                node.as_mut().payload = payload;
                ret = node;
                break
            };

            node_opt = child_node;
        };

        ret
    }

}

impl<T, P> Node<T, P> {
    pub fn head(value: NonNull<T>, payload: P) -> NonNull<Self> {
        to_heap(Self {
            parent: zero_node_ptr(),
            left: zero_node_ptr(),
            right: zero_node_ptr(),
            color: Color::Black,
            value,
            payload,
        })
    }

    fn black(value: NonNull<T>, parent: NonNull<Self>, payload: P) -> NonNull<Self> {
        to_heap(Self {
            left: zero_node_ptr(),
            right: zero_node_ptr(),
            color: Color::Black,
            parent: Some(parent),
            value,
            payload,
        })
    }

    fn red(value: NonNull<T>, parent: NonNull<Self>, payload: P) -> NonNull<Self> {
        to_heap(Self {
            left: zero_node_ptr(),
            right: zero_node_ptr(),
            color: Color::Red,
            parent: Some(parent),
            value,
            payload,
        })
    }

    unsafe fn grandparent(&self) -> NodePtr<T, P> {
        self.parent?.as_ref().parent
    }

    unsafe fn sibling(&self) -> NodePtr<T, P> {
        if self.is_left() {
            self.parent?.as_ref().right
        } else {
            self.parent?.as_ref().left
        }
    }

    unsafe fn uncle(&self) -> NodePtr<T, P> {
        self.parent?.as_ref().sibling()
    }

    //   ancestor
    //    \
    //     self
    //      \
    //      right
    //      /
    //      l
    //------------
    //   ancestor
    //    \
    //     self        right
    //      \
    //       l
    //------------
    //   right
    //    \
    //     self        ancestor
    //      \
    //       l
    //------------
    //   ancestor
    //    \
    //     right
    //    /
    //   self
    //    \
    //     l
    //------------
    unsafe fn rotate_left(&mut self) -> Result<(), ()> {
        dbg!("rotate_left", self as *const _);
        let mut ancestor = self.parent;
        let mut right = self.right.ok_or(())?;
        self.right = right.as_ref().left;
        right.as_mut().left.map(|mut x| {
            x.as_mut().parent = node_ptr(self);
        });

        right.as_mut().left = node_ptr(self);
        right.as_mut().parent = ancestor;
        ancestor.map(|mut x| {
            x.as_mut()
                .replace_child(right, NonNull::new_unchecked(self))
                .expect("rotate_left: An ancestor is not related to self")
        });

        self.parent = Some(right);
        Ok(())
    }

    fn is(&self, other: NonNull<Self>) -> bool {
        unsafe {
            other.as_ref() as *const _ == self as *const _
        }
    }

    fn is_left(&self) -> bool {
        let parent = self.parent.unwrap();
        let is_left = unsafe { parent.as_ref().left.map(|x| self.is(x)) };
        let is_right = unsafe { parent.as_ref().right.map(|x| false) };
        let side = is_left.or(is_right);
        return side.expect("fail to get if a node is the left leg");
    }

    unsafe fn replace_child(&mut self, new: NonNull<Self>, old: NonNull<Self>) -> Result<(), ()> {
        if old.as_ref().is_left() {
            self.left = Some(new);
        } else {
            self.right = Some(new);
        }

        Ok(())
    }

    unsafe fn rotate_right(&mut self) -> Result<(), ()> {
        dbg!("rotate_right", self as *const _);
        let mut ancestor = self.parent;
        let mut left = self.left.take().ok_or(())?;
        self.left = left.as_mut().right;
        left.as_mut().right.map(|mut x| {
            x.as_mut().parent = node_ptr(self);
        });

        left.as_mut().right = node_ptr(self);
        left.as_mut().parent = ancestor;
        ancestor.map(|mut x| {
            x.as_mut()
                .replace_child(left, NonNull::new_unchecked(self))
                .expect("rotate_left: An ancestor is not related to self")
        });

        self.parent = Some(left);
        Ok(())
    }

    /// return the descendant node of pointed one which has the minimum.
    unsafe fn min_node(&self) -> NodePtr<T, P> {
        let mut left = self.left;
        while let Some(x) = left.and_then(|x| x.as_ref().left) {
            left = Some(x)
        }

        left
    }

    unsafe fn find_replacement(&self) -> NodePtr<T, P> {
        match self.left.and(self.right) {
            Some(right) => right.as_ref().min_node().or(Some(right)),
            None => self.left.or(self.right)
        }
    }

    unsafe fn unlink_parent(&mut self) {
        self.parent.map(|mut x| {
            if self.is_left() {
                x.as_mut().left = None;
            } else {
                x.as_mut().right = None;
            }
        });
    }

    // case 1: self doesn't have any child -> so it's a leaf
    // case 2: self has both children -> so we go to the node which is the smallest node in its
    // right branch. The node is a leaf or it's a node with a right child which is a leaf.
    // case 3: self is root and it has only one child (because of case 2) -> so it's easy to get
    // the replacement value and remove the replacement.
    // case 4: self is internal node and it has only right child (because case 2) the child is
    // the replacement and a leaf (because of case 2) -> so self can be replaced by the child
    // without caring of its children
    unsafe fn del_step(&mut self) -> State<T, P> {
        let replacement = self.find_replacement();
        if replacement.is_none() {
            if self.color.is_black() {
                self.fix_double_black()
            } else {
                self.sibling().map(|mut x| x.as_mut().color = Color::Red);
            }

            self.unlink_parent();
            // unallocate(NonNull::new_unchecked(self as *mut _));
            return State::Stop(NonNull::new_unchecked(self as *mut _));
        }

        let mut replacement = replacement.unwrap();
        if self.left.and(self.right).is_some() {
            mem::swap(&mut self.value, &mut replacement.as_mut().value);
            // self.value = replacement.as_ref().value;
            mem::swap(&mut self.payload, &mut replacement.as_mut().payload);
            return State::Continue(replacement);
        }

        if self.parent.is_none() {
            mem::swap(&mut self.value, &mut replacement.as_mut().value);
            // self.value = replacement.as_ref().value;
            mem::swap(&mut self.payload, &mut replacement.as_mut().payload);
            self.left = None;
            self.right = None;
            // unallocate(replacement);
            return State::Stop(replacement);
        }

        let mut parent = self.parent;
        replacement.as_mut().parent = parent;
        parent.map(|mut x| x.as_mut().replace_child(
            replacement, NonNull::new_unchecked(self as *mut _)
        ));

        let are_both_black = self.color.is_black() && replacement.as_ref().color.is_black();
        if are_both_black {
            replacement.as_mut().fix_double_black();
        } else {
            replacement.as_mut().color = Color::Black;
        }

        // unallocate(NonNull::new_unchecked(self as *mut _));
        return State::Stop(NonNull::new_unchecked(self as *mut _));
    }

    // Note 1: should return head for a tree and a value for a resouce allocator.
    // Note 2: maximum number of iterations is 2. It's only one for a leaf or for a node with one
    // child. 2 iterations for a node which has 2 children. In this case the node is replaced by
    // the node which is most left child (has minimum value) in its left subtree.
    pub unsafe fn del(&mut self) -> NonNull<Node<T, P>> {
        let mut ptr = State::Continue(NonNull::new_unchecked(self as *mut _));
        // let ret = self.value;
        // let payload = self.payload;
        loop {
            match ptr {
                State::Continue(mut node) => ptr = node.as_mut().del_step(),
                State::Stop(node) => return node,
            }
        }
    }

    unsafe fn fix_double_black_step(&mut self) -> NodePtr<T, P> {
        // it's impossible to be a black node without sibling because it violates property 5 (about
        // equal number of black nodes from a given node to its descendant leafs). The check look
        // like redudant. It seems impossible to call the function for a red node. Only a red node
        // can be without a sibling.
        let mut sibling = match self.sibling() {
            Some(x) => x,
            None => return self.parent,
        };

        //   p(b)    |    s(b)
        //  /    \   |   /
        // n(b)  s(r)|  p(r)
        //           | /
        //           |n(b)
        let mut parent = self.parent.unwrap();
        if sibling.as_ref().color.is_red() {
            // parent is black because because the sibling is red. If the parent would be a red
            // then the sibling and the node are red.
            sibling.as_mut().color = Color::Black;
            parent.as_mut().color = Color::Red;
            if self.is_left() {
                parent.as_mut().rotate_left();
            } else {
                parent.as_mut().rotate_right();
            }

            return node_ptr(self);
        }

        if let Some(Color::Red) = sibling.as_ref().left.map(|x| x.as_ref().color) {
            let mut sibling_left = sibling.as_ref().left.unwrap();
            if sibling.as_ref().is_left() {
                // initial (left left)
                //       p(?)
                //      /    \
                //    s(b)   n(b) x
                //   /   \
                // sl(r) sr(r) // sr can't be a black node because it violates property 5.
                // p(r)-s(b)-sr(b) but p(r)-n(b). So it means that one branch of p has more blacks
                // than another. It doesn't matter what color the parent.
                // -----------------
                //    s(?)
                //   /    \
                // sl(b)  p(b)
                //       /    \
                //      sr(r) n(b) x
                sibling_left.as_mut().color = Color::Black;
                sibling.as_mut().color = parent.as_ref().color;
                parent.as_mut().rotate_right();
            } else {
                // initial (right left)
                //    p(?)
                //   /    \
                // n(b)   s(b)
                //       /    \
                //      sl(r) sr(r)
                // -----------------
                //       p(?)
                //      /    \
                //    n(b)   sl(?)
                //             \    
                //              s(b)
                //               \
                //                sr(r)
                //----------------------
                //      sl(?)
                //     /    \
                //    p(b)   s(b)
                //   /        \    
                //  n(b) x     sr(r)
                sibling_left.as_mut().color = parent.as_ref().color;
                sibling.as_mut().rotate_right();
                parent.as_mut().rotate_left();
            }

            parent.as_mut().color = Color::Black;
            return None;
        }

        if let Some(Color::Red) = sibling.as_ref().right.map(|x| x.as_ref().color) {
            let mut sibling_right = sibling.as_ref().right.unwrap();
            if sibling.as_ref().is_left() {
                sibling_right.as_mut().color = parent.as_ref().color;
                sibling.as_mut().rotate_left();
                parent.as_mut().rotate_right();
            } else {
                sibling_right.as_mut().color = Color::Black;
                sibling.as_mut().color = parent.as_ref().color;
                parent.as_mut().rotate_left();
            }

            parent.as_mut().color = Color::Black;
            return None;
        }

        sibling.as_mut().color = Color::Red;
        return match parent.as_ref().color {
            Color::Black => Some(parent),
            Color::Red => {
                parent.as_mut().color = Color::Black;
                None
            }
        }
    }

    // Note 1: Sibling of the node is black.
    // Note 2: Self is a leaf node (without children). It can be irrelevant for the second
    // iteration.
    unsafe fn fix_double_black(&mut self) {
        let mut node = node_ptr(self);
        while let Some(_) = node.and_then(|x| x.as_ref().parent) {
            node = node.unwrap().as_mut().fix_double_black_step()
        }
    }

}

// new is root => make it black
// new's parent is black => nothing to do
// new's parent is red and new's uncle is red => make them black, make grandparent red; repeat for
// the grandparent.
// new's parent is red and new's uncle is black or there is no uncle =>
//    g(b)
//   /
//  p(r)
//   \
//    n(r)
//  - if new node is the right leg of its parent and the parent is the left leg of grandparent =>
//      rotate parent to left to get straight branch: g-n-p. Next actions are for the parent node.
//  g(b)
//   \
//    p(r)
//   /
//  n(r)
//  - else if new node is the left leg of its parent and the parent is the right leg of grandparent =>
//      rotate parent to right to get straight branch: g-n-p. Next actions are for the parent node.
//  - if the new node is left leg of its parent then rotate the grandparent to right.
//  - else if the new node is right leg of its parent then rotate the grandparent to left.
//  - make the parent black and the grandparent red
pub unsafe fn repair<T: Ord, P>(new: NonNull<Node<T, P>>) {
    let mut node = Some(new);
    while node.is_some() {
        node = repair_step(node.take().unwrap());
    }
}

unsafe fn repair_step<T: Ord, P>(mut new: NonNull<Node<T, P>>) -> NodePtr<T, P> {
    if new.as_ref().parent.is_none() {
        new.as_mut().color = Color::Black;
        return None;
    }

    let mut parent = new.as_ref().parent.clone().unwrap();
    if parent.as_ref().color.is_black() {
        return None;
    }

    let mut grandparent = new.as_ref().grandparent().clone().unwrap();
    if let Some(ref mut uncle) = new.as_ref().uncle() {
        if uncle.as_ref().color.is_red() {
            parent.as_mut().color = Color::Black;
            uncle.as_mut().color = Color::Black;
            grandparent.as_mut().color = Color::Red;
            return Some(grandparent);
        }
    }

    //    grandparent
    //   /
    //  parent
    //   \
    //    new
    if Some(new) == parent.as_ref().right && Some(parent) == grandparent.as_ref().left {
        parent.as_mut().rotate_left().unwrap();
        mem::swap(&mut parent, &mut new);
    //  grandparent
    //   \
    //    parent
    //   /
    //  new
    } else if Some(new) == parent.as_ref().left && Some(parent) == grandparent.as_ref().right {
        parent.as_mut().rotate_right();
        mem::swap(&mut parent, &mut new);
    }

    //  grandparent |     g
    //   \          |    /
    //    parent    |   p
    //     \        |  /
    //      new     | n
    if Some(new) == parent.as_ref().left {
        grandparent.as_mut().rotate_right().unwrap();
    } else if Some(new) == parent.as_ref().right {
        grandparent.as_mut().rotate_left().unwrap();
    } else {
        panic!("unexpected state of nodes");
    }

    parent.as_mut().color = Color::Black;
    grandparent.as_mut().color = Color::Red;

    return None;
}
