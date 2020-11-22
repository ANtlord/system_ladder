use super::insertionsort::insertionsort;

// TODO: shuffling to guarantee performance.
pub fn quicksort<T, F: Fn(&T, &T) -> bool>(myslice: &mut [T], less: F) {
    actual_quicksort(myslice, &less);
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
