use std::ptr::NonNull;
use crate::utils::vector::Swim;
use crate::utils::vector::Sink;
use crate::utils::vector::SwimSink;
use std::collections::VecDeque;
use std::ops::Deref;
use std::mem;
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Ref;
use std::ops::Div;
use std::ops::Sub;

mod indexed_heap;
#[cfg(test)]
mod tests;

type FnBox<T> = Box<dyn Fn(&T, &T) -> bool>;

pub trait Compare<T> {
    fn compare(&self, _: &T, _: &T) -> bool;
}

pub trait Index<T> {
    fn index(&self, _: &T, pos: usize) -> bool;
}

pub struct Heap<T, SW> {
    data: Vec<T>,
    sw: SW,
}

impl<T: PartialOrd, SW> Heap<T, SW> {
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

impl<T: PartialOrd> Heap<T, SwimSink<FnBox<T>>> {
    fn max() -> Self {
        Self{
            data: Vec::new(),
            sw: SwimSink(Box::new(|a, b| a < b)),
        }
    }

    fn min() -> Self {
        Self{
            data: Vec::new(),
            sw: SwimSink(Box::new(|a, b| a > b)),
        }
    }

    fn new(predicate: FnBox<T>) -> Self {
        Self{ 
            data: Vec::new(),
            sw: SwimSink(predicate),
        }
    }
}

impl<T: PartialOrd, SW: Swim<T> + Sink<T>> Heap<T, SW> {
    pub fn push(&mut self, value: T) {
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
        self.sw.swim(&mut self.data, pos);
    }

    fn sink(&mut self, mut pos: usize) {
        self.sw.sink(&mut self.data, pos);
    }
}
