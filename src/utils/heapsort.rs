use std::fmt;

use super::vector::sink;

pub fn heapsort<T: fmt::Debug, F: Fn(&T, &T) -> bool>(data: &mut [T], less: F) {
    for i in (0 ..= data.len() / 2).rev() {
        sink(data, i, &less);
    }

    for i in (1 .. data.len()).rev() {
        data.swap(0, i);
        sink(&mut data[..i], 0, &less);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time;
    use crate::random::xorshift_rng as random;

    fn must_sorted(v: Vec<u32>) {
        let res = v.iter().enumerate().skip(1).find(|(i, e)| e < &&v[i - 1]);
        if let Some((index, e)) = res {
            assert!(false, "{} > {}. Index = {}", e, v[index - 1], index);
        }
    }

    fn heapsort_random_set(count: usize) -> Vec<u32> {
        let mut ret: Vec<u32> = vec![0; count].into_iter().map(|_| random()).collect();
        heapsort(&mut ret, |x, y| x < y);
        ret
    }

    #[test]
    fn odd_reverse_sorted_elements() {
        let mut v = vec![3,2,1];
        heapsort(&mut v, |a, b| a < b);
        assert_eq!(v, vec![1,2,3]);
    }

    #[test]
    fn even_reverse_sorted_elements() {
        let mut v = vec![4,3,2,1];
        heapsort(&mut v, |x, y| x < y);
        assert_eq!(v, vec![1,2,3,4]);
    }

    #[test]
    fn random_5000() {
        must_sorted(heapsort_random_set(5000));
    }

    #[test]
    fn random_1000() {
        must_sorted(heapsort_random_set(1000));
    }

    #[test]
    fn random_100() {
        must_sorted(heapsort_random_set(100));
    }
}
