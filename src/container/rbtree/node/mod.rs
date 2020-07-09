use std::cmp::Ord;
use std::collections::BTreeMap;
use std::fmt;
use std::mem;
use std::ptr::NonNull;

#[cfg(test)]
mod tests;

pub type NodePtr<T> = Option<NonNull<Node<T>>>;

fn zero_node_ptr<T: Ord>() -> NodePtr<T> {
    None
}

unsafe fn node_ptr<T: Ord>(reference: &mut Node<T>) -> NodePtr<T> {
    Some(NonNull::new_unchecked(reference))
}

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

pub struct Node<T> {
    pub parent: NodePtr<T>,
    pub left: NodePtr<T>,
    pub right: NodePtr<T>,
    pub color: Color,
    pub value: NonNull<T>,
}

fn to_heap<T>(val: T) -> NonNull<T> {
    unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(val))) }
}

impl<T: Ord> Node<T> {
    pub fn head(value: NonNull<T>) -> NonNull<Self> {
        to_heap(Self {
            parent: zero_node_ptr(),
            left: zero_node_ptr(),
            right: zero_node_ptr(),
            color: Color::Black,
            value,
        })
    }

    fn black(value: NonNull<T>, parent: NonNull<Self>) -> NonNull<Self> {
        to_heap(Self {
            left: zero_node_ptr(),
            right: zero_node_ptr(),
            color: Color::Black,
            parent: Some(parent),
            value,
        })
    }

    fn red(value: NonNull<T>, parent: NonNull<Self>) -> NonNull<Self> {
        to_heap(Self {
            left: zero_node_ptr(),
            right: zero_node_ptr(),
            color: Color::Red,
            parent: Some(parent),
            value,
        })
    }

    unsafe fn grandparent(&self) -> NodePtr<T> {
        self.parent?.as_ref().parent
    }

    unsafe fn sibling(&self) -> NodePtr<T> {
        let parent = self.parent?;
        let ref parent = parent.as_ref();
        let left = parent.left?.as_ref() as *const _;
        let right = parent.right?.as_ref() as *const _;
        let selfptr = self as *const _;
        if left == selfptr {
            parent.right
        } else if right == selfptr {
            parent.left
        } else {
            panic!("Parent of a node is not a left or right node of the parent");
        }
    }

    unsafe fn uncle(&self) -> NodePtr<T> {
        self.parent?.as_ref().sibling()
    }

    pub unsafe fn add(&mut self, val: NonNull<T>) -> NonNull<Self> {
        let mut parent = NonNull::new_unchecked(self);
        let mut value = Some(val);
        while value.is_some() {
            let val = value.take().unwrap();
            let mut parent_clone = parent.clone();
            let mut node = if val.as_ref() < parent.as_ref().value.as_ref() {
                Some(&mut parent_clone.as_mut().left)
            } else if val.as_ref() > parent.as_ref().value.as_ref() {
                Some(&mut parent_clone.as_mut().right)
            } else {
                None
            };

            if let Some(ref mut x) = node {
                value = Node::try_set_leg(&mut parent, x, val);
            }
        }

        parent
    }

    unsafe fn try_set_leg(
        parent: &mut NonNull<Self>,
        node: &mut NodePtr<T>,
        value: NonNull<T>,
    ) -> Option<NonNull<T>> {
        match node {
            Some(ref mut x) => {
                *parent = NonNull::new_unchecked(x.as_mut());
                Some(value)
            }
            None => {
                let new = Self::red(value, *parent);
                node.replace(new);
                *parent = new;
                None
            }
        }
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

        self.parent = Some(right);
        right.as_mut().left = node_ptr(self);
        right.as_mut().parent = ancestor;
        ancestor.map(|mut x| {
            x.as_mut()
                .replace_child(right, NonNull::new_unchecked(self))
                .expect("rotate_left: An ancestor is not related to self")
        });

        Ok(())
    }

    unsafe fn replace_child(&mut self, new: NonNull<Self>, old: NonNull<Self>) -> Result<(), ()> {
        let aleft = &mut self.left;
        let aright = &mut self.right;
        if aleft == &Some(old) {
            *aleft = Some(new);
            Ok(())
        } else if aright == &Some(old) {
            *aright = Some(new);
            Ok(())
        } else {
            Err(())
        }
    }

    unsafe fn rotate_right(&mut self) -> Result<(), ()> {
        let mut ancestor = self.parent;
        let mut left = self.left.take().ok_or(())?;
        self.left = left.as_mut().right;
        left.as_mut().right.map(|mut x| {
            x.as_mut().parent = node_ptr(self);
        });

        self.parent = Some(left);
        left.as_mut().right = node_ptr(self);
        left.as_mut().parent = ancestor;
        ancestor.map(|mut x| {
            x.as_mut()
                .replace_child(left, NonNull::new_unchecked(self))
                .expect("rotate_left: An ancestor is not related to self")
        });
        Ok(())
    }
}

// new is root => make it black
// new's parent is black => nothing to do
// new's parent is red and new's uncle is red => make them black, make grandparent red; repeat for
// the grandparent.
// new's parent is red and new's uncle is black or there is no uncle =>
//    g
//   /
//  p
//   \
//    n
//  - if new node is the right leg of its parent and the parent is the left leg of grandparent =>
//      rotate parent to left to get straight branch: g-n-p. Next actions are for the parent node.
//  g
//   \
//    p
//   /
//  n
//  - else if new node is the left leg of its parent and the parent is the right leg of grandparent =>
//      rotate parent to right to get straight branch: g-n-p. Next actions are for the parent node.
//  - if the new node is left leg of its parent then rotate the grandparent to right.
//  - else if the new node is right leg of its parent then rotate the grandparent to left.
//  - make the parent black and the grandparent red
pub unsafe fn repair<T: Ord>(new: NonNull<Node<T>>) {
    let mut node = Some(new);
    while node.is_some() {
        node = repair_step(node.take().unwrap());
    }
}

unsafe fn repair_step<T: Ord>(mut new: NonNull<Node<T>>) -> NodePtr<T> {
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

    if Some(new) == parent.as_ref().left {
        grandparent.as_mut().rotate_right();
    } else if Some(new) == parent.as_ref().right {
        grandparent.as_mut().rotate_left().unwrap();
    } else {
        panic!("unexpected state of nodes");
    }

    parent.as_mut().color = Color::Black;
    grandparent.as_mut().color = Color::Red;

    return None;
}
