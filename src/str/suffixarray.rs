use std::convert::AsRef;
use std::clone::Clone;

use crate::utils::msd;
use crate::utils::quicksort;

struct SuffixArray<'a> {
    data: &'a str,
    suffixes: Vec<Suffix<'a>>,
    prefixes: Vec<usize>,
}

#[derive(Debug, PartialEq)]
struct Suffix<'a>(usize, &'a str);

impl<'a> AsRef<[u8]> for Suffix<'a> {
    fn as_ref(&self) -> &[u8] {
        self.1.as_ref()
    }
}

impl<'a> Clone for Suffix<'a> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

// TODO: Linear-Time Longest-Common-Prefix Computation in Suffix Arrays and Its Applications.
fn lcp<T: AsRef<[u8]>>(input: &[T]) -> Vec<usize> {
    input.iter().skip(1).take(input.len() - 1).zip(input.iter()).map(|(current, previous)| {
        let mut count = 0;

        loop {
            if count >= current.as_ref().len() && count >= previous.as_ref().len() {
                break count
            }

            if current.as_ref()[count] != previous.as_ref()[count] {
                break count
            }

            count += 1;
        }
    }).collect()
}

impl<'a> SuffixArray<'a> {
    // TODO: Replace MSD by Manber & Myers algorithm.
    // TODO: Linear Suffix Array Construction by Almost Pure Induced-Sorting
    // TODO: https://gist.github.com/sumanth232/e1600b327922b6947f51
    fn new(data: &'a str) -> Self {
        let mut suffixes: Vec<_> = (0 .. data.len()).into_iter()
            .map(|x| Suffix(x, &data[x .. ])).collect();

        msd(&mut suffixes);
        let prefixes = lcp(&suffixes);
        debug_assert_eq!(prefixes.len() + 1, suffixes.len());
        Self {
            prefixes,
            suffixes,
            data,
        }
    }
}

mod tests {
    use super::*;

    static DELIMETER: u8 = 0x01;

    mod longest_common_substring {
        use super::*;

        #[test]
        fn basic() {
            // TODO: how to do it without copying.
            let word1 = "abcab".to_owned();
            let word2 = "acd".to_owned();
            let word2_starts_at = word1.len();
            let mut input = word1;

            input.push_str(&word2);
            let sa = SuffixArray::new(input.as_ref());
            assert_eq!(sa.suffixes, &[
                Suffix(3, "abacd"),
                Suffix(0, "abcabacd"),
                Suffix(5, "acd"),

                Suffix(4, "bacd"),
                Suffix(1, "bcabacd"),

                Suffix(2, "cabacd"),
                Suffix(6, "cd"),

                Suffix(7, "d"),
            ]);

            let mut lcp: Vec<_> = sa.prefixes.iter().cloned().enumerate().collect();
            assert_eq!(&lcp, &[
                (0, 2),
                (1, 1),
                (2, 0),
                (3, 1),
                (4, 0),
                (5, 1),
                (6, 0),
            ]);

            quicksort(&mut lcp, |x, y| x.1 > y.1);
            assert_eq!(&lcp, &[
                (0, 2),
                (3, 1),
                (5, 1),
                (1, 1),
                (2, 0), 
                (6, 0),
                (4, 0),
            ]);

            let (mut from, mut to) = (0, 0);
            for (i, prefix_length) in lcp {
                let left = &sa.suffixes[i];
                let right = &sa.suffixes[i + 1];
                if (
                    left.0 < word2_starts_at && right.0 < word2_starts_at
                    || left.0 >= word2_starts_at && right.0 >= word2_starts_at
                ) {
                    continue;
                }

                from = left.0;
                to = left.0 + prefix_length;
                break;
            }

            assert_eq!(from, 2);
            assert_eq!(to, 3);
            // should be "a" but "c" is here because of quicksort is not stable.
            assert_eq!(&sa.data[from .. to], "c");
        }
    }
}
