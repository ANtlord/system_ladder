use std::marker::PhantomData;
use super::Heap;
use crate::utils::vector::Swim;
use crate::utils::vector::Sink;
use crate::utils::vector::Predicate;
use crate::container::rbtree::Tree;
use super::FnBox;
use std::cmp::Ordering;
use std::cell::RefCell;
use std::rc::Rc;
use std::mem;
use std::borrow::Borrow;

struct Pair<K, V>(K, Rc<V>);

impl<K: PartialEq, V> PartialEq for Pair<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K: PartialOrd, V> PartialOrd for Pair<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

struct SwimSink<P, C>(P, C);

impl<T, P: Fn(&T, &T) -> bool, C: Fn(&T, &T)> Sink<T> for SwimSink<P, C>{
    fn swap(&self, data: &mut [T], from: usize, to: usize) {
        data.swap(from, to);
        (self.1)(&data[from], &data[to]);
    }
}

impl<T, P: Fn(&T, &T) -> bool, C: Fn(&T, &T)> Swim<T> for SwimSink<P, C>{
    fn swap(&self, data: &mut [T], from: usize, to: usize) {
        data.swap(from, to);
        (self.1)(&data[from], &data[to]);
    }
}

impl<T, P: Fn(&T, &T) -> bool, C: Fn(&T, &T)> Predicate<T> for SwimSink<P, C> {
    fn predicate(&self, left: &T, right: &T) -> bool {
        (self.0)(left, right)
    }
}

// trait Index<K> {
//     fn get(&self, key: K) -> usize;
//     fn insert(&self, key: K, value: usize);
// }

pub struct IndexedHeap<K, V, SW, I> {
    heap: Heap<Pair<K, V>, SW>,
    // index: Vec<usize>, point to a place in the `heap`
    keys: I, // point to a place in the `index`
    // Idea is finding a corresponding index in `index` through keys as keys provides index by
    // place in the heap
}

type PairSwimSink<K, V, C> = SwimSink<FnBox<Pair<K, V>>, C>;
type SwimSinkCallback<K, V> = Box<dyn Fn(&Pair<K, V>, &Pair<K, V>)>;
type TreePtr<V> = Rc<RefCell<Tree<Rc<V>, usize>>>;

// TODO: figure out design without referece counters of values.
// The container doesn't seem a data owner but a support container.
impl<K: PartialOrd, V: 'static + Ord> IndexedHeap<K, V, PairSwimSink<K, V, SwimSinkCallback<K, V>>, TreePtr<V>> {
    pub fn new(predicate: FnBox<Pair<K, V>>) -> Self {
        let tree: TreePtr<V> = Rc::new(RefCell::new(Tree::default()));
        let tree_clone = tree.clone();
        Self {
            heap: Heap{
                data: Vec::new(),
                sw: SwimSink(predicate, Box::new(move |left: &Pair<K, V>, right: &Pair<K, V>| {
                    tree_clone.borrow_mut().swap(&left.1, &right.1);
                })),
            },
            keys: tree,
        }
    }

    /// key - priority
    /// value - an object the priority for
    pub fn insert(&mut self, key: K, value: V) {
        let value = Rc::new(value);
        self.keys.borrow_mut().insert(value.clone(), self.heap.len());
        let pair = Pair(key, value.clone());
        self.heap.push(pair);
    }

    pub fn pop(&mut self) -> Option<(K, V)> {
        let pair: Pair<K, V> = self.heap.pop()?;
        let index = self.keys.borrow_mut().remove(&pair.1)?;
        drop(index);
        let value = match Rc::try_unwrap(pair.1) {
            Ok(x) => x,
            Err(e) => panic!("fail popping from indexed binary heap. Popped item has {} references", Rc::strong_count(&e)),
        };

        Some((pair.0, value))
    }

    pub fn change_key(&mut self, key: &V, priority: K) -> Result<(), ()> {
        let tree: &RefCell<Tree<Rc<V>, usize>> = self.keys.borrow();
        let index = *tree.borrow().get(key).ok_or(())?;
        self.heap.data[index].0 = priority;
        self.heap.swim(index);
        self.heap.sink(index);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod create {
        use super::*;

        #[test]
        fn max_oriented() {
            let mut indexed_heap = IndexedHeap::new(Box::new(|x, y| x < y));
            let persons = vec![(0.1, "Jack".to_owned()), (0.3, "Alex".to_owned()), (0.5, "Jane".to_owned())];
            persons.iter().for_each(|(x, y)| indexed_heap.insert(x.clone(), y.clone()));
            // Jane, Alex, Jack
            persons.into_iter().rev().for_each(|pair| assert_eq!(indexed_heap.pop(), Some(pair)));
            assert_eq!(indexed_heap.pop(), None);
        }

        #[test]
        fn min_oriented() {
            let mut indexed_heap = IndexedHeap::new(Box::new(|x, y| x > y));
            let persons = vec![(0.1, "Jack".to_owned()), (0.3, "Alex".to_owned()), (0.5, "Jane".to_owned())];
            persons.iter().for_each(|(x, y)| indexed_heap.insert(x.clone(), y.clone()));
            // Jack, Alex, Jane
            persons.into_iter().for_each(|pair| assert_eq!(indexed_heap.pop(), Some(pair)));
            assert_eq!(indexed_heap.pop(), None);
        }

        #[test]
        fn reverse_insert() {
            let mut indexed_heap = IndexedHeap::new(Box::new(|x, y| x > y));
            let persons = vec![(0.1, "Jack".to_owned()), (0.3, "Alex".to_owned()), (0.5, "Jane".to_owned())];
            persons.iter().rev().for_each(|(x, y)| indexed_heap.insert(x.clone(), y.clone()));
            assert_eq!((*indexed_heap.keys).borrow().items().len(), 3);
            persons.into_iter().for_each(|pair| assert_eq!(indexed_heap.pop(), Some(pair)));
            assert_eq!(indexed_heap.pop(), None);
        }
    }

    mod change_key {
        use super::*;
        use crate::tprintln;

        #[test]
        fn basic() {
            assert!(false);
            let mut indexed_heap = IndexedHeap::new(Box::new(|x, y| x < y)); // max oriented
            let mut persons = vec![(0.1, "Jack".to_owned()), (0.3, "Alex".to_owned()), (0.5, "Jane".to_owned())];
            persons.iter().for_each(|(x, y)| indexed_heap.insert(x.clone(), y.clone()));
            dbg!(indexed_heap.heap.data.iter().map(|x| x.1.as_ref().clone()).collect::<Vec<String>>());
            persons[1].0 = 0.6;
            let alex = &persons[1];
            println!("change {} to {}", &alex.1, &alex.0);
            indexed_heap.change_key(&alex.1, alex.0);
            assert_eq!((*indexed_heap.keys).borrow().items().len(), 3);
            persons.swap(1, 2);
            persons.into_iter().rev().for_each(|pair| assert_eq!(indexed_heap.pop(), Some(pair)));
        }
    }
}
