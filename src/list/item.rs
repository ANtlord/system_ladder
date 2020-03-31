use std::ptr::null_mut;
// use std::ptr::NonNull;
use std::cell::Cell;
use std::cmp::PartialEq;
use std::mem::replace;

#[derive(Debug)]
pub struct Link<T>(Cell<*mut Item<T>>);

impl<T> Link<T> {
    pub fn new(value: T) -> Self {
        Link(Cell::new(Box::into_raw(Box::new(Item::new(value)))))
    }

    pub fn null() -> Self {
        Link(Cell::new(null_mut()))
    }

    pub fn is_null(&self) -> bool {
        self.get_ptr().is_null()
    }

    pub fn get_mut(&mut self) -> Option<&mut Item<T>> {
        if self.get_ptr().is_null() {
            None
        } else {
            unsafe { Some(&mut *self.get_ptr()) }
        }
    }

    pub fn get(&self) -> Option<&Item<T>> {
        if self.get_ptr().is_null() {
            None
        } else {
            unsafe { Some(&mut *self.get_ptr()) }
        }
    }

    pub fn get_ptr(&self) -> *mut Item<T> {
        self.0.get()
    }

    pub fn get_value(&self) -> Option<&T> {
        if self.get_ptr().is_null() {
            None
        } else {
            unsafe { Some(&(*self.get_ptr()).get()) }
        }
    }

    pub fn from_link(l: &Link<T>) -> Self {
        Link(Cell::new(l.get_ptr()))
    }

    pub fn add_next(&mut self, value: T) -> Result<Link<T>, &str> {
        let res = match self.get_mut() {
            Some(x) => {
                x.add_next(value);
                Ok(Self::from_link(&x.next))
            }
            None => Err("Pointer is null"),
        };
        if res.is_ok() {
            self.get_mut().unwrap().next.get_mut().unwrap().prev = Link::from_link(&self);
        }
        res
    }

    pub fn add_prev(&mut self, value: T) -> Result<Link<T>, &str> {
        let res = match self.get_mut() {
            Some(x) => {
                x.add_prev(value);
                Ok(Self::from_link(&x.prev))
            }
            None => Err("Pointer is null"),
        };
        if res.is_ok() {
            self.get_mut().unwrap().prev.get_mut().unwrap().next = Link::from_link(&self);
        }
        res
    }

    pub fn remove(mut self) -> Option<T> {
        if self.get_ptr().is_null() {
            None
        } else {
            Some(unsafe {
                let ptr = Box::from_raw(self.get_ptr());
                (*ptr).remove()
            })
        }
    }

    pub fn next_mut(&mut self) -> Option<&mut Link<T>> {
        Some(&mut self.get_mut()?.next)
    }

    pub fn next(&self) -> Option<&Link<T>> {
        Some(&self.get()?.next)
    }

    pub fn prev(&self) -> Option<&Link<T>> {
        Some(&self.get()?.prev)
    }
}

impl<T> PartialEq for Link<T> {
    fn eq(&self, other: &Link<T>) -> bool {
        self.0 == other.0
    }
}

#[derive(Debug)]
pub struct Item<T> {
    value: T,
    prev: Link<T>,
    next: Link<T>,
}

impl<T> Item<T> {
    fn new(value: T) -> Self {
        Item {
            value: value,
            prev: Link::null(),
            next: Link::null(),
        }
    }

    fn add_next(&mut self, value: T) {
        let mut old_next = replace(&mut self.next, Link::new(value));
        if old_next.get_mut().is_none() {
            return;
        }
        old_next.get_mut().unwrap().prev = Link::from_link(&self.next)
    }

    fn add_prev(&mut self, value: T) {
        let mut old_prev = replace(&mut self.prev, Link::new(value));
        if old_prev.get_mut().is_none() {
            return;
        }
        old_prev.get_mut().unwrap().next = Link::from_link(&self.prev)
    }

    fn remove(mut self) -> T {
        let mut prev = replace(&mut self.prev, Link::null());
        let mut next = replace(&mut self.next, Link::null());
        if prev.get_mut().is_some() {
            prev.get_mut().unwrap().next = Link::from_link(&next);
        }
        if next.get_mut().is_some() {
            next.get_mut().unwrap().prev = Link::from_link(&prev);
        }
        self.value
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_eq() {
        let l1 = Link::new(1);
        let l2 = Link::from_link(&l1);
        assert!(l1 == l2);
    }

    #[test]
    fn link_null_eq() {
        let l1: Link<u8> = Link::null();
        let l2: Link<u8> = Link::null();
        assert!(l1 == l2);
    }
}
