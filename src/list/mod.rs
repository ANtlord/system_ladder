// use self::item::Link;
//
mod item;
mod iter;

use std::mem::replace;
use self::item::Link;
use self::iter::Iter;
use self::iter::Cursor;

pub struct List<T> {
    pub head: Link<T>,
    tail: Link<T>,
}

// FIXME: implement drop
// TODO: implement iter_mut
// TODO: implement len
impl<T> List<T> {
    pub fn cursor(&mut self) -> Cursor<T> {
        return Cursor::new(&self.head)
    }

    pub fn iter(&self) -> Iter<T> {
        return Iter::new(&self.head)
    }

    pub fn new() -> Self {
        List{head: Link::null(), tail: Link::null()}
    }

    pub fn is_empty(&self) -> bool {
        self.tail.is_null()
    }

    fn create_first(&mut self, value: T) {
        self.head = Link::new(value);
        self.tail = Link::from_link(&self.head);
    }

    pub fn push_back(&mut self, value: T) {
        if self.is_empty() {
            self.create_first(value);
        } else {
            let res = self.tail.add_next(value).unwrap();
            replace(&mut self.tail, res);
        }
    }

    pub fn push_front(&mut self, value: T) {
        if self.is_empty() {
            self.create_first(value);
        } else {
            let res = self.head.add_prev(value).unwrap();
            replace(&mut self.head, res);
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            println!("is empty");
            None
        } else {
            let old_tail = replace(&mut self.tail, Link::null());
            if old_tail != self.head {
                self.tail = Link::from_link(old_tail.prev().unwrap());
            } else {
                self.head = Link::null();
            }
            old_tail.remove()
        }
    }
    
    fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            println!("is empty");
            None
        } else {
            let old_head = replace(&mut self.head, Link::null());
            if old_head != self.tail {
                self.head = Link::from_link(old_head.next().unwrap());
            } else {
                self.tail = Link::null();
            }
            old_head.remove()
        }
    }

    pub fn back(&self) -> Option<&T> {
        self.tail.get_value()
    }
    
    pub fn front(&self) -> Option<&T> {
        self.head.get_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_first() {
        let mut list = List::new();
        assert!(list.front().is_none());
        assert!(list.back().is_none());

        list.push_back(123);
        assert!(list.front().unwrap() == &123);
        assert!(list.back().unwrap() == &123);
    }

    #[test]
    fn push_back() {
        let mut list = List::new();
        list.push_back(000);
        list.push_back(111);

        assert!(list.front().unwrap() == &000);
        assert!(list.back().unwrap() == &111);
    }

    #[test]
    fn push_front() {
        let mut list = List::new();
        list.push_front(000);
        list.push_front(111);

        assert!(list.front().unwrap() == &111);
        assert!(list.back().unwrap() == &000);
    }

    #[test]
    fn pop_front() {
        let mut list = List::new();
        list.push_front(000);
        list.push_front(111);

        let val = list.pop_front().unwrap();
        assert!(val == 111, "val == {}", val);
        let val = list.pop_front().unwrap();
        assert!(val == 000, "val == {}", val);
    }

    #[test]
    fn pop_back() {
        let mut list = List::new();
        list.push_back(000);
        list.push_back(111);

        let val = list.pop_back().unwrap();
        assert!(val == 111, "val == {}", val);
        let val = list.pop_back().unwrap();
        assert!(val == 000, "val == {}", val);
    }

    #[test]
    fn iter_empty() {
        let list: List<u8> = List::new();
        let iter = list.iter();
        let mut count = 0;
        for i in iter {
            count += 1;
        }
        assert_eq!(count, 0);
    }

    #[test]
    fn iter_filled() {
        let mut list: List<u8> = List::new();
        list.push_back(111);
        list.push_back(222);
        list.push_front(000);

        assert_eq!(list.iter().count(), 3);
        assert_eq!(list.iter().collect::<Vec<&u8>>(), vec![&000, &111, &222]);
    }

    //#[test]
    //fn insert_next() {
    //    let mut list: List<u8> = List::new();
    //    list.push_back(111);
    //    list.push_back(222);
    //    list.push_front(000);

    //    let mut iter = list.iter_mut();
    //    assert_eq!(iter.next(), Some(&mut 111));
    //    iter.insert_next(66);
    //    assert_eq!(iter.next(), Some(&mut 66));
    //    assert_eq!(iter.next(), Some(&mut 222));
    //}
}
