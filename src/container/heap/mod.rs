use std::ptr::NonNull;

pub struct Heap<T> {
    data: Vec<T>,
    predicate: Box<dyn Fn(&T, &T) -> bool>,
}

// max oriented heap
impl<T: Ord> Heap<T> {
    fn max() -> Self {
        Self{data: Vec::new(), predicate: Box::new(|a, b| a > b)}
    }

    fn min() -> Self {
        Self{data: Vec::new(), predicate: Box::new(|a, b| a < b)}
    }

    fn push(&mut self, value: T) {
        self.data.push(value);
        self.swim(self.data.len() - 1);
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }

    fn reserve(&mut self, v: usize) {
        self.data.reserve(v)
    }

    fn front(&self) -> &T {
        &self.data[0]
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

    fn ifind(&self, from: usize, val: &T) -> Option<usize> {
        if from > self.data.len() {
            None
        } else if &self.data[from - 1] == val {
            Some(from - 1)
        } else {
            self.ifind(2 * from, val).or(self.ifind(2 * from + 1, val))
        }
    }

    fn find(&self, val: &T) -> Option<usize> {
        self.ifind(1, val)
    }

    fn take(&mut self, val: &T) -> Option<T> {
        let pos = self.find(val)?;
        Some(self.remove(pos))
    }

    fn remove(&mut self, pos: usize) -> T {
        let ret = self.data.swap_remove(pos);
        self.sink(pos);
        self.swim(pos);
        ret
    }

    fn swim(&mut self, mut pos: usize) {
        pos += 1;
        let predicate = &self.predicate;
        while pos > 1 && predicate(&self.data[pos - 1], &self.data[pos / 2 - 1]) {
            self.data.swap(pos - 1, pos / 2 - 1);
            pos /= 2;
        }
    }

    fn sink(&mut self, mut pos: usize) {
        pos += 1;
        let predicate = &self.predicate;
        while pos * 2 < self.data.len() + 1 {
            let mut next = pos * 2;
            if next < self.data.len() && predicate(&self.data[next], &self.data[next - 1]) {
                next += 1;
            }
            
            if predicate(&self.data[pos - 1], &self.data[next - 1]) {
                break;
            }

            self.data.swap(pos - 1, next - 1);
            pos = next;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn gt() -> Box<dyn Fn(&u32, &u32) -> bool> {
        Box::new(|a, b| a > b)
    }

    fn lt() -> Box<dyn Fn(&u32, &u32) -> bool> {
        Box::new(|a, b| a < b)
    }

    #[test]
    fn swim() {
        let mut h = Heap{data: vec![1, 2], predicate: gt()};
        h.swim(1);
        assert_eq!(h.data, vec![2, 1]);

        let mut h = Heap{data: vec![1, 2, 3], predicate: gt()};
        h.swim(2);
        assert_eq!(h.data, vec![3, 2, 1]);

        let mut h = Heap{data: vec![80, 20, 30, 40, 50, 60, 70], predicate: gt()};
        h.swim(6);
        assert_eq!(h.data, vec![80, 20, 70, 40, 50, 60, 30]);
    }

    #[test]
    fn sink() {
        let mut h = Heap{data: vec![1, 2], predicate: gt()};
        h.sink(0);
        assert_eq!(h.data, vec![2, 1]);

        let mut h = Heap{data: vec![1, 2, 3], predicate: gt()};
        h.sink(0);
        assert_eq!(h.data, vec![3, 2, 1]);

        let mut h = Heap{data: vec![10, 20, 30, 40, 50, 60, 70], predicate: gt()};
        h.sink(0);
        assert_eq!(h.data, vec![30, 20, 70, 40, 50, 60, 10]);
    }

    #[test]
    fn remove() {
        let mut h = Heap{data: vec![80, 79, 70, 78, 77, 60, 30, 76], predicate: gt()};
        let res = h.remove(2);
        assert_eq!(res, 70);
        assert_eq!(h.data, vec![80, 79, 76, 78, 77, 60, 30]);

        let mut h = Heap{data: vec![80, 79, 70, 78, 77, 60, 30, 20], predicate: gt()};
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
        let mut last = h.pop().unwrap();
        while let Some(x) = h.pop() {
            assert!(last > x);
            last = x;
        }
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

    struct DynamicMedian {
        lt: Heap<u16>,
        gt: Heap<u16>,
    }

    impl DynamicMedian {
        fn new(len: usize) -> Self {
            let len = len / 2 * 2 + 1;
            let mut lt = Heap::max();
            let mut gt = Heap::min();
            lt.reserve(len / 2);
            gt.reserve(len / 2 + 1);
            Self{lt, gt}
        }
    }

    #[test]
    fn dymanic_median() {
        let d = DynamicMedian::new(100);
    }
}
