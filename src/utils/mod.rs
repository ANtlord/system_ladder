pub mod vector;

use std::mem;
use std::ptr;
use std::fmt;


fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        mem::swap(&mut left, &mut right);
        right %= left;
    }
    left
}

pub fn insertionsort<T, F: Fn(&T, &T) -> bool>(myslice: &mut [T], less: F) {
    let len = myslice.len();
    for i in 0 .. len {
        for j in (1 ..= i).rev() {
            if less(&myslice[j], &myslice[j - 1]) {
                myslice.swap(j, j - 1);
            }
        }
    }
}

fn actual_quicksort<T, F: Fn(&T, &T) -> bool>(myslice: &mut [T], less: &F) {
    let len = myslice.len();
    if len < 2 {
        return;
    }

    if len < 5 {
        return insertionsort(myslice, less);
    }

    let mut i = 0;
    let mut lt = 0;
    let mut gt = len - 1;
    while i <= gt {
        if less(&myslice[i], &myslice[lt]) {
            myslice.swap(i, lt);
            lt += 1;
            i += 1;
        } else if less(&myslice[lt], &myslice[i]) {
            myslice.swap(i, gt);
            gt -= 1;
        } else {
            i += 1;
        }
    }

    actual_quicksort(&mut myslice[0 .. lt], less);
    actual_quicksort(&mut myslice[gt + 1 .. len], less);
}

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

// TODO: shuffling to guarantee performance.
pub fn quicksort<T, F: Fn(&T, &T) -> bool>(myslice: &mut [T], less: F) {
    actual_quicksort(myslice, &less);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time;

    fn random() -> u32 {
        let v = time::SystemTime::now().duration_since(time::UNIX_EPOCH)
            .unwrap().as_nanos() as u32;
        let mut random = v;
        random ^= random << 13;
        random ^= random >> 17;
        random << 5
    }


    fn must_sorted(v: Vec<u32>) {
        let res = v.iter().enumerate().skip(1).find(|(i, e)| e < &&v[i - 1]);
        if let Some((index, e)) = res {
            assert!(false, "{} > {}. Index = {}", e, v[index - 1], index);
        }
    }

    mod quicksort {
        use super::*;

        #[test]
        fn odd_elements() {
            let mut v = vec![3,2,1];
            quicksort(&mut v, |x, y| x < y);
            assert_eq!(v, vec![1,2,3]);
        }

        #[test]
        fn even_elements() {
            let mut v = vec![4,3,1,1];
            quicksort(&mut v, |x, y| x < y);
            assert_eq!(v, vec![1,1,3,4]);
        }

        #[test]
        fn single_element() {
            let mut v = vec![4];
            quicksort(&mut v, |x, y| x < y);
            assert_eq!(v, vec![4]);
        }

        #[test]
        fn two_element() {
            let mut v = vec![4,2];
            quicksort(&mut v, |x, y| x < y);
            assert_eq!(v, vec![2, 4]);
        }

        #[test]
        fn equal_elements() {
            let mut v = vec![4, 4, 2, 1, 0, 8, 7, 7];
            quicksort(&mut v, |x, y| x < y);
            assert_eq!(v, vec![0, 1, 2 ,4, 4, 7, 7, 8]);
        }

        #[test]
        fn equal_elements_with_pivot_duplicate() {
            let mut v = vec![4, 4, 2, 1, 1, 8, 9, 10];
            quicksort(&mut v, |x, y| x < y);
            assert_eq!(v, vec![1, 1, 2, 4, 4, 8, 9, 10]);
        }

        #[test]
        fn two_sorted_elements() {
            let mut v = vec![1, 2];
            quicksort(&mut v, |x, y| x < y);
            assert_eq!(v, vec![1, 2]);
        }

        fn quicksort_random_set(count: usize) -> Vec<u32> {
            let mut ret: Vec<u32> = vec![0; count].into_iter().map(|_| random()).collect();
            quicksort(&mut ret, |x, y| x < y);
            ret
        }

        #[test]
        fn random_5000() {
            must_sorted(quicksort_random_set(5000));
        }

        #[test]
        fn random_1000() {
            must_sorted(quicksort_random_set(1000));
        }

        #[test]
        fn random_100() {
            must_sorted(quicksort_random_set(100));
        }
    }

    mod insertion {
        use super::*;

        #[test]
        fn odd_reverse_sorted_elements() {
            let mut v = vec![3,2,1];
            insertionsort(&mut v, |a, b| a < b);
            assert_eq!(v, vec![1,2,3]);
        }

        fn insertionsort_random_set(count: usize) -> Vec<u32> {
            let mut ret: Vec<u32> = vec![0; count].into_iter().map(|_| random()).collect();
            insertionsort(&mut ret, |x, y| x < y);
            ret
        }

        #[test]
        fn random_500() {
            must_sorted(insertionsort_random_set(500));
        }

        #[test]
        fn random_250() {
            must_sorted(insertionsort_random_set(250));
        }

        #[test]
        fn random_100() {
            must_sorted(insertionsort_random_set(100));
        }
    }

    mod merge {
        use super::*;

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

        struct Droplet(i32, *mut u32);
        impl Drop for Droplet {
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
                Droplet(20, &mut drop_count as *mut _),
                Droplet(80, &mut drop_count as *mut _),
                Droplet(30, &mut drop_count as *mut _),
                Droplet(40, &mut drop_count as *mut _),
                Droplet(10, &mut drop_count as *mut _),
                Droplet(60, &mut drop_count as *mut _),
                Droplet(90, &mut drop_count as *mut _),
                Droplet(70, &mut drop_count as *mut _),
                Droplet(50, &mut drop_count as *mut _),
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
}
