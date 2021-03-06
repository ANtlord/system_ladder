/// Moves elements have a field equals to sample.
///
/// # Examples
///
/// ```
/// let mut numbers = vec![0, 1, 1, 0, 3, 2, 0];
/// move_to_end_by(&0, &mut numbers, |x| &x);
/// let len = numbers.len();
/// assert_eq!(numbers[len - 1], 0);
/// assert_eq!(numbers[len - 2], 0);
/// assert_eq!(numbers[len - 3], 0);
/// ```
pub fn move_to_end_by<T, F>(sample: &T, heap: &mut [T], are_equal: F) -> Option<usize>
where
    F: Fn(&T, &T) -> bool,
{
    let heap_len = heap.len();
    match heap_len {
        1 if are_equal(&sample, &heap[0]) => Some(0),
        1 | 0 => None,
        _ => {
            let mut count = heap_len;

            for i in 0..count {

                if !are_equal(&sample, &heap[i]) {
                    continue;
                }

                while count == heap_len || are_equal(&sample, &heap[count]) && count > i {
                    count -= 1;
                }

                heap.swap(i, count);

            }

            Some(count)
        }
    }
}

pub trait Predicate<T> {
    fn predicate(&self, _: &T, _: &T) -> bool;
}

pub trait Swim<T>: Predicate<T> {
    fn swap(&self, data: &mut [T], from: usize, to: usize) {
        data.swap(from, to);
    }

    fn swim(&self, data: &mut [T], mut pos: usize) {
        pos += 1;
        while pos > 1 && self.predicate(&data[pos / 2 - 1], &data[pos - 1]) {
            self.swap(data, pos - 1, pos / 2 - 1);
            pos /= 2;
        }
    }
}

pub struct SwimSink<P>(pub P);

impl<T, P: Fn(&T, &T) -> bool> Sink<T> for SwimSink<P>{}

impl<T, P: Fn(&T, &T) -> bool> Swim<T> for SwimSink<P>{}

impl<T, P: Fn(&T, &T) -> bool> Predicate<T> for SwimSink<P> {
    fn predicate(&self, left: &T, right: &T) -> bool {
        (self.0)(left, right)
    }
}


/// swim...
pub fn swim<T>(data: &mut [T], mut pos: usize, predicate: impl Fn(&T, &T) -> bool) {
    let sw = SwimSink(predicate);
    sw.swim(data, pos);
}

pub trait Sink<T>: Predicate<T> {
    fn swap(&self, data: &mut [T], from: usize, to: usize) {
        data.swap(from, to);
    }

    fn sink(&self, data: &mut [T], mut pos: usize) {
        pos += 1;
        while pos * 2 < data.len() + 1 {
            let mut next = pos * 2;
            if next < data.len() && self.predicate(&data[next - 1], &data[next]) {
                next += 1;
            }

            if !self.predicate(&data[pos - 1], &data[next - 1]) {
                break;
            }

            data.swap(pos - 1, next - 1);
            pos = next;
        }
    }
}

/// Swaps nth element and its child (2nth or 2nth + 1 element) as long as an element and its child
/// satisfy `predicate`. nth elements and 2nth + 1 element are swapped if 2nth + 1 and 2nth
/// elements satisfies the same `predicate` otherwise nth and 2nth elements are swapped.
pub fn sink<T>(data: &mut [T], mut pos: usize, predicate: impl Fn(&T, &T) -> bool) {
    let sw = SwimSink(predicate);
    sw.sink(data, pos);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_with_duplicates() {
        let mut numbers = vec![0, 1, 1, 0, 3, 2, 0];
        let since = move_to_end_by(&0, &mut numbers, |x, y| x == y);
        assert_eq!(since.unwrap(), 4);
    }

    #[test]
    fn sample_no_duplicates() {
        let mut numbers = vec![0, 1, 1, 0, 3, 2, 0];
        let since = move_to_end_by(&26, &mut numbers, |x, y| x == y);
        assert_eq!(since.unwrap(), numbers.len());
    }

    #[test]
    fn odds_more_than_evens() {
        let mut numbers = vec![0, 0, 0, 0, 3, 2, 0];
        let since = move_to_end_by(&0, &mut numbers, |x, y| x == y);
        assert_eq!(since.unwrap(), 2);
        assert_eq!(numbers[since.unwrap() .. ], [0, 0, 0, 0, 0]);
    }

    fn lt(a: &u32, b: &u32) -> bool {
        a < b
    }

    #[test]
    fn test_sink() {
        let mut data = vec![1, 2];
        sink(&mut data, 0, lt);
        assert_eq!(data, vec![2, 1]);

        let mut data = vec![1, 2, 3];
        sink(&mut data, 0, lt);
        assert_eq!(data, vec![3, 2, 1]);

        let mut data = vec![10, 20, 30, 40, 50, 60, 70];
        sink(&mut data, 0, lt);
        assert_eq!(data, vec![30, 20, 70, 40, 50, 60, 10]);
    }

    #[test]
    fn test_swim() {
        let mut ve = vec![1, 2];
        swim(&mut ve, 1, lt);
        assert_eq!(ve, vec![2, 1]);

        let mut ve = vec![1, 2, 3];
        swim(&mut ve, 2, lt);
        assert_eq!(ve, vec![3, 2, 1]);

        let mut ve = vec![80, 20, 30, 40, 50, 60, 70];
        swim(&mut ve, 6, lt);
        assert_eq!(ve, vec![80, 20, 70, 40, 50, 60, 30]);
    }
}
