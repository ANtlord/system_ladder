use std::ptr::NonNull;
use crate::utils::vector::sink;
use crate::utils::vector::swim;
use std::collections::VecDeque;
use std::ops::Deref;
use std::mem;
use std::marker::PhantomData;
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Ref;
use std::ops::Div;
use std::ops::Sub;

mod indexed_heap;

type FnBox<T> = Box<dyn Fn(&T, &T) -> bool>;

pub trait Compare<T> {
    fn compare(&self, _: &T, _: &T) -> bool;
}

enum Predicate<'a, T> {
    Owned(FnBox<T>),
    Borrowed(&'a dyn Fn(&T, &T) -> bool),
}

pub struct Heap<'a, T> {
    data: Vec<T>,
    predicate: Predicate<'a, T>,
}

impl<'a, T: PartialOrd> Heap<'a, T> {
    fn len(&self) -> usize {
        self.data.len()
    }

    fn reserve(&mut self, v: usize) {
        self.data.reserve(v)
    }

    fn front(&self) -> &T {
        &self.data[0]
    }

    fn find_step(&self, from: usize, val: &T) -> Option<usize> {
        if from > self.data.len() {
            None
        } else if &self.data[from - 1] == val {
            Some(from - 1)
        } else {
            self.find_step(2 * from, val).or(self.find_step(2 * from + 1, val))
        }
    }

    fn find(&self, val: &T) -> Option<usize> {
        self.find_step(1, val)
    }

}

impl<'a, T: PartialOrd> Heap<'a, T> {
    fn max() -> Self {
        Self{data: Vec::new(), predicate: Predicate::Owned(Box::new(|a, b| a < b))}
    }

    fn min() -> Self {
        Self{data: Vec::new(), predicate: Predicate::Owned(Box::new(|a, b| a > b))}
    }

    // fn new(predicate: &'a dyn Fn(&T, &T) -> bool) -> Self {
    fn new(predicate: FnBox<T>) -> Self {
        let data = Vec::new();
        Self{ data, predicate: Predicate::Owned(predicate) }
    }

    // fn new(predicate: FnBox<T>) -> Self {
    //     let data = Vec::new();
    //     Self{ data, predicate: Predicate::Owned(predicate) }
    // }

    fn push(&mut self, value: T) {
        self.data.push(value);
        self.swim(self.data.len() - 1);
    }
    
    fn pop(&mut self) -> Option<T> {
        if self.data.len() == 0 {
            None
        } else {
            let ret = self.data.swap_remove(0);
            self.sink(0);
            Some(ret)
        }
    }

    fn take(&mut self, val: &T) -> Option<T> {
        let pos = self.find(val)?;
        Some(self.remove(pos))
    }

    fn remove(&mut self, pos: usize) -> T {
        let ret = self.data.swap_remove(pos);
        if pos < self.data.len() {
            self.sink(pos);
            self.swim(pos);
        }

        ret
    }

    fn swim(&mut self, mut pos: usize) {
        match &self.predicate {
            Predicate::Owned(func) => swim(&mut self.data, pos, &*func),
            Predicate::Borrowed(func) => swim(&mut self.data, pos, &*func),
        }
    }

    fn sink(&mut self, mut pos: usize) {
        match &self.predicate {
            Predicate::Owned(func) => sink(&mut self.data, pos, &*func),
            Predicate::Borrowed(func) => sink(&mut self.data, pos, &*func),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gt<'a>() -> Predicate<'a, u32> {
        Predicate::Owned(Box::new(|a, b| a > b))
    }

    fn lt<'a>() -> Predicate<'a, u32> {
        Predicate::Owned(Box::new(|a, b| a < b))
    }

    #[test]
    fn swim() {
        let mut h = Heap{data: vec![1, 2], predicate: lt()};
        h.swim(1);
        assert_eq!(h.data, vec![2, 1]);

        let mut h = Heap{data: vec![1, 2, 3], predicate: lt()};
        h.swim(2);
        assert_eq!(h.data, vec![3, 2, 1]);

        let mut h = Heap{data: vec![80, 20, 30, 40, 50, 60, 70], predicate: lt()};
        h.swim(6);
        assert_eq!(h.data, vec![80, 20, 70, 40, 50, 60, 30]);
    }

    #[test]
    fn remove() {
        let mut h = Heap{data: vec![80, 79, 70, 78, 77, 60, 30, 76], predicate: lt()};
        let res = h.remove(2);
        assert_eq!(res, 70);
        assert_eq!(h.data, vec![80, 79, 76, 78, 77, 60, 30]);

        let mut h = Heap{data: vec![80, 79, 70, 78, 77, 60, 30, 20], predicate: lt()};
        let res = h.remove(2);
        assert_eq!(res, 70);
        assert_eq!(h.data, vec![80, 79, 60, 78, 77, 20, 30]);
    }

    #[test]
    fn push_heap_max() {
        let mut h = Heap::max();
        h.push(10);
        h.push(20);
        h.push(30);
        assert_eq!(h.data, vec![30, 10, 20]);
        assert_eq!(h.pop(), Some(30));
        assert_eq!(h.pop(), Some(20));
        assert_eq!(h.pop(), Some(10));
        assert_eq!(h.pop(), None);
    }

    #[test]
    fn push_heap_min() {
        let mut h = Heap::min();
        h.push(10);
        h.push(20);
        h.push(30);
        assert_eq!(h.data, vec![10, 20, 30]);
        let mut last = h.pop().unwrap();
        while let Some(x) = h.pop() {
            assert!(last < x);
            last = x;
        }
    }

    mod dynamic_median {
        use super::*;

        struct DynamicMedian<T> {
            data: Rc<RefCell<Vec<T>>>,
            queue: VecDeque<usize>,
            lt: Heap<'static, usize>,
            gt: Heap<'static, usize>,
            len: usize,
        }

        trait Avg<In = Self> {
            type Out;

            fn avg(slice: &[&In]) -> Self::Out;
        }

        impl Avg for f32 {
            type Out = f32;

            fn avg(slice: &[&f32]) -> Self::Out {
                slice.iter().map(|x| *x).sum::<f32>() / slice.len() as f32
            }
        }

        impl Avg for u32 {
            type Out = u32;

            fn avg(slice: &[&u32]) -> Self::Out {
                slice.iter().map(|x| *x).sum::<u32>() / slice.len() as u32
            }
        }

        impl<T: Avg<Out=T> + 'static> DynamicMedian<T> {
            fn get(&self) -> T {
                let data = self.data.borrow();
                if self.gt.len() == self.lt.len() {
                    T::avg(&[
                        &data[*self.gt.front()],
                        &data[*self.lt.front()],
                    ])
                    // data[*self.gt.front()].avg(&data[*self.lt.front()])
                } else {
                    T::avg(&[&data[*self.gt.front()]])
                }
            }
        }

        impl<T: PartialOrd + 'static> DynamicMedian<T> {
            fn new(len: usize) -> Self {
                let len = len / 2 * 2 + 1;
                let data = Rc::new(RefCell::new(Vec::new()));
                let heaplt = {
                    let dt = Rc::clone(&data);
                    let mut ret = Heap::new(Box::new(move |x, y| dt.borrow()[*x] < dt.borrow()[*y]));
                    ret.reserve(len / 2);
                    ret
                };

                let heapgt = {
                    let dt = Rc::clone(&data);
                    let mut ret = Heap::new(Box::new(move |x, y| dt.borrow()[*x] > dt.borrow()[*y]));
                    ret.reserve(len / 2 + 1);
                    ret
                };

                Self{
                    len,
                    data,
                    queue: VecDeque::new(),
                    lt: heaplt,
                    gt: heapgt,
                }
            }

            fn is_full(&self) -> bool {
                self.data.borrow().len() == self.len
            }

            fn items(&self) -> Ref<Vec<T>> {
                Ref::map(self.data.borrow(), |t| t)
            }

            fn at(&self, index: usize) -> Ref<T> {
                Ref::map(self.data.borrow(), |t| &t[index])
            }

            fn add(&mut self, val: T) {
                let index = if self.is_full() {
                    let to_remove = {
                        let data = self.data.borrow();
                        let to_remove = self.queue.pop_front().unwrap();
                        if &data[to_remove] <= &data[*self.lt.front()] {
                            self.lt.take(&to_remove);
                        } else {
                            self.gt.take(&to_remove);
                        }

                        to_remove
                    };

                    self.data.borrow_mut()[to_remove] = val;
                    to_remove
                } else {
                    self.data.borrow_mut().push(val);
                    self.data.borrow().len() - 1
                };

                self.queue.push_back(index);
                let data = self.data.borrow();
                if self.lt.len() > 0 && &data[index] < &data[*self.lt.front()] {
                    self.lt.push(index);
                } else {
                    self.gt.push(index);
                }
                drop(data);

                self.rebalance();
            }

            fn rebalance(&mut self) {
                if self.gt.len() > self.lt.len() + 1 {
                    let index = self.gt.pop().unwrap();
                    self.lt.push(index);
                }

                if self.lt.len() > self.gt.len() {
                    let index = self.lt.pop().unwrap();
                    self.gt.push(index);
                }
            }
        }

        #[test]
        fn rebalance_in_empty() {
            let mut d = DynamicMedian::new(100);
            for i in 0 .. 5 {
                d.add(i);
            }

            assert_eq!(d.lt.len(), 2);
            assert_eq!(d.gt.len(), 3);
        }

        #[test]
        fn add() {
            let mut d = DynamicMedian::new(100);
            d.add(0.1);
            assert_eq!(*d.at(*d.gt.front()), 0.1);

            d.add(0.9);
            dbg!(&d.data);
            assert_eq!(*d.at(*d.gt.front()), 0.9);
            assert_eq!(*d.at(*d.lt.front()), 0.1);

            d.add(0.8);
            assert_eq!(d.lt.len(), 1);
            assert_eq!(d.gt.len(), 2);
            dbg!(&d.data, &d.gt.data, &d.lt.data);
            assert_eq!(*d.at(*d.gt.front()), 0.8);
            assert_eq!(*d.at(*d.lt.front()), 0.1);

            d.add(0.2);
            assert_eq!(*d.at(*d.gt.front()), 0.8);
            assert_eq!(*d.at(*d.lt.front()), 0.2);
        }

        #[test]
        fn rebalance_in_full() {
            fn test(count: usize) {
                let mut d = DynamicMedian::new(count);
                d.add(1.1);
                d.add(2.2);
                d.add(3.3);
                d.add(4.4);
                d.add(5.5);
                assert_eq!(d.lt.len(), 5.min(d.len) / 2);
                assert_eq!(d.gt.len(), 5.min(d.len) / 2 + 1);
                (0 .. 1000).for_each(|i| d.add(i as f32));
                assert_eq!(d.lt.len(), count / 2);
                assert_eq!(d.gt.len(), count / 2 + 1);
            }

            test(3);
            test(4);
            test(5);
            test(99);
            test(100);
            test(1001);
        }

        #[test]
        fn get() {
            let mut d = DynamicMedian::new(3);
            (0 .. 3).for_each(|i| d.add(i as f32));
            let median = d.get();
            dbg!(&d.data);
            assert_eq!(median, 1f32);

            let mut d = DynamicMedian::new(101);
            (0 .. 26).for_each(|i| d.add(i as f32));
            assert_eq!(d.get(), 12.5);

            (0 .. 101).for_each(|i| d.add(i as f32));
            assert_eq!(d.get(), 50f32);

            (101 .. 201).for_each(|i| d.add(i as f32));
            assert_eq!(d.get(), 150f32);

            let mut d = DynamicMedian::new(101);
            (0 .. 26).rev().for_each(|i| d.add(i as f32));
            assert_eq!(d.get(), 12.5);
        }
    }
}
