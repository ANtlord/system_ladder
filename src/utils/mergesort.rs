use std::mem;
use std::ptr;

use super::insertionsort::insertionsort;

pub fn _merge<T, F: Fn(&T, &T) -> bool>(myslice: &mut [T], less: &F) {
    if myslice.len() < 20 {
        insertionsort(myslice, less);
        return;
    }

    let mid = myslice.len() / 2;
    let mut buf = Vec::with_capacity(mid);
    let mut myslice_ptr = myslice.as_mut_ptr();
    let mut bufptr = buf.as_mut_ptr();
    unsafe {
        ptr::copy_nonoverlapping(myslice_ptr, bufptr, mid);
        buf.set_len(mid);
    }

    let mut left_cursor = 0;
    let mut right_cursor = mid;
    let mut i = 0;
    while left_cursor < mid && right_cursor < myslice.len() {
        if less(&buf[left_cursor], &myslice[right_cursor]) {
            unsafe {
                ptr::copy_nonoverlapping(bufptr.add(left_cursor), myslice_ptr.add(i), 1);
            }

            left_cursor += 1;
        } else {
            myslice.swap(i, right_cursor);
            right_cursor += 1;
        }

        i += 1;
    }

    if left_cursor < mid {
        unsafe {
            ptr::copy_nonoverlapping(bufptr.add(left_cursor), myslice_ptr.add(i), mid - left_cursor);
        }
    }

    unsafe {
        // TODO: to prevent drops of T. Find a better way.
        // Look closer to merge_sort in standard library or Rustonomicon
        let _: Vec<u8> = mem::transmute(buf);
    }
}

pub fn _mergesort<T, F: Fn(&T, &T) -> bool>(myslice: &mut [T], less: &F) {
    let len = myslice.len();
    let mid = len / 2;
    if len < 2 {
        return;
    }

    _mergesort(&mut myslice[..mid], less);
    _mergesort(&mut myslice[mid..], less);
    _merge(myslice, less);
}

pub fn mergesort<T, F: Fn(&T, &T) -> bool>(myslice: &mut [T], less: F) {
    _mergesort(myslice, &less);
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::xorshift_rng as random;

    fn must_sorted(v: Vec<u32>) {
        let res = v.iter().enumerate().skip(1).find(|(i, e)| e < &&v[i - 1]);
        if let Some((index, e)) = res {
            assert!(false, "{} > {}. Index = {}", e, v[index - 1], index);
        }
    }

    #[test]
    fn odd_reverse_sorted_elements() {
        let mut v = vec![3,2,1];
        mergesort(&mut v, |a, b| a < b);
        assert_eq!(v, vec![1,2,3]);
    }

    #[test]
    fn even_reverse_sorted_elements() {
        let mut v = vec![4,3,2,1];
        mergesort(&mut v, |x, y| x < y);
        assert_eq!(v, vec![1,2,3,4]);
    }

    #[test]
    fn equal_elements() {
        let mut v = vec![4, 4, 2, 1, 0, 8, 7, 7];
        mergesort(&mut v, |x, y| x < y);
        assert_eq!(v, vec![0, 1, 2 ,4, 4, 7, 7, 8]);
    }

    struct DropCount(i32, *mut u32);
    impl Drop for DropCount {
        fn drop(&mut self) {
            unsafe {
                (*self.1) += 1;
            }
        }
    }

    #[test]
    fn no_drops() {
        let mut drop_count = 0u32;
        let mut v = vec![
            DropCount(20, &mut drop_count as *mut _),
            DropCount(80, &mut drop_count as *mut _),
            DropCount(30, &mut drop_count as *mut _),
            DropCount(40, &mut drop_count as *mut _),
            DropCount(10, &mut drop_count as *mut _),
            DropCount(60, &mut drop_count as *mut _),
            DropCount(90, &mut drop_count as *mut _),
            DropCount(70, &mut drop_count as *mut _),
            DropCount(50, &mut drop_count as *mut _),
        ];

        mergesort(&mut v, |x, y| x.0 < y.0);
        assert_eq!(drop_count, 0);
    }

    fn mergesort_random_set(count: usize) -> Vec<u32> {
        let mut ret: Vec<u32> = vec![0; count].into_iter().map(|_| random()).collect();
        mergesort(&mut ret, |x, y| x < y);
        ret
    }

    #[test]
    fn random_5000() {
        must_sorted(mergesort_random_set(5000));
    }

    #[test]
    fn random_1000() {
        must_sorted(mergesort_random_set(1000));
    }

    #[test]
    fn random_100() {
        must_sorted(mergesort_random_set(100));
    }
}
