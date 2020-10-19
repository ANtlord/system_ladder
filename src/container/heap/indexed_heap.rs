use std::marker::PhantomData;
use super::Heap;
use crate::utils::vector::Swim;
use crate::utils::vector::Sink;
use crate::utils::vector::Predicate;
use crate::container::rbtree::Tree;
use super::FnBox;

pub struct IndexedHeap<T, SW> {
    heap: Heap<T, SW>,
}

struct SwimSink<P, C>(P, C);

impl<T, P: Fn(&T, &T) -> bool, C: Fn(usize, usize)> Sink<T> for SwimSink<P, C>{
    fn swap(&self, data: &mut [T], from: usize, to: usize) {
        data.swap(from, to);
        (self.1)(from, to);
    }
}

impl<T, P: Fn(&T, &T) -> bool, C: Fn(usize, usize)> Swim<T> for SwimSink<P, C>{}

impl<T, P: Fn(&T, &T) -> bool, C: Fn(usize, usize)> Predicate<T> for SwimSink<P, C> {
    fn predicate(&self, left: &T, right: &T) -> bool {
        (self.0)(left, right)
    }
}

impl<T> IndexedHeap<T, SwimSink<FnBox<T>, Box<dyn Fn(usize, usize)>>> {
    pub fn new(predicate: FnBox<T>) -> Self {
        Self {
            heap: Heap{
                data: Vec::new(),
                sw: SwimSink(predicate, Box::new(|x, y| ())),
            }
        }
    }
}
