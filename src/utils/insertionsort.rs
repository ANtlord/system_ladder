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
