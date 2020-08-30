pub mod vector;

use std::mem;

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        mem::swap(&mut left, &mut right);
        right %= left;
    }
    left
}

fn insertionsort<T, F: Fn(&T, &T) -> bool>(myslice: &mut [T], less: F) {
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

    if len < 20 {
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

    // myslice.swap(i, last);
    actual_quicksort(&mut myslice[0 .. lt], less);
    actual_quicksort(&mut myslice[gt + 1 .. len], less);
}

// TODO: shuffling to guarantee performance.
pub fn quicksort<T, F: Fn(&T, &T) -> bool>(myslice: &mut [T], less: F) {
    actual_quicksort(myslice, &less);
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    mod insertion {
        use super::*;

        #[test]
        fn qweasd() {
            let mut v = vec![3,2,1];
            insertionsort(&mut v, |a, b| a < b);
            assert_eq!(v, vec![1,2,3]);
        }
    }
}
