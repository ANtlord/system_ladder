pub mod vector;

use std::mem;

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        mem::swap(&mut left, &mut right);
        right %= left;
    }
    left
}

fn actual_quicksort<T, F: Fn(&T, &T) -> bool>(myslice: &mut [T], is_left_greater_right: &F) {
    let len = myslice.len();
    if len < 2 {
        return;
    }

    let last = len - 1;
    let mut i = 0;
    {
        for j in 0..last {
            if !is_left_greater_right(&myslice[j], &myslice[last]) {
                myslice.swap(i, j);
                i += 1;
            }
        }
    }
    myslice.swap(i, last);

    actual_quicksort(&mut myslice[0 .. i], is_left_greater_right);
    actual_quicksort(&mut myslice[i + 1 .. len], is_left_greater_right);
}

pub fn quicksort<T, F: Fn(&T, &T) -> bool>(myslice: &mut [T], is_left_greater_right: F) {
    actual_quicksort(myslice, &is_left_greater_right);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odd_elements() {
        let mut v = vec![3,2,1];
        quicksort(&mut v, |x, y| x > y);
        assert_eq!(v, vec![1,2,3]);
    }

    #[test]
    fn even_elements() {
        let mut v = vec![4,3,1,1];
        quicksort(&mut v, |x, y| x > y);
        assert_eq!(v, vec![1,1,3,4]);
    }

    #[test]
    fn single_element() {
        let mut v = vec![4];
        quicksort(&mut v, |x, y| x > y);
        assert_eq!(v, vec![4]);
    }

    #[test]
    fn two_element() {
        let mut v = vec![4,2];
        quicksort(&mut v, |x, y| x > y);
        assert_eq!(v, vec![2, 4]);
    }

    #[test]
    fn equal_elements() {
        let mut v = vec![4, 4, 2, 1, 0, 8, 7, 7];
        quicksort(&mut v, |x, y| x > y);
        assert_eq!(v, vec![0, 1, 2 ,4, 4, 7, 7, 8]);
    }

    #[test]
    fn equal_elements_with_pivot_duplicate() {
        let mut v = vec![4, 4, 2, 1, 1, 8, 9, 10];
        quicksort(&mut v, |x, y| x > y);
        assert_eq!(v, vec![1, 1, 2, 4, 4, 8, 9, 10]);
    }

    #[test]
    fn two_sorted_elements() {
        let mut v = vec![1, 2];
        quicksort(&mut v, |x, y| x > y);
        assert_eq!(v, vec![1, 2]);
    }
}
