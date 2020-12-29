/// Find the longest palindromic substring with Manacher's algorithm in linear time.

//P[i] = P[i'], iff (max-i) > P[i']
//P[i]>=(max-i), iff (max-i) <= P[i']
//P[i]=min(P[i'], max-i) when the 3rd palindrome is not extendable past max
//P[i] = 0, iff max-i < 0
//
//
//Case 1: P[i] = P[i'],  iff (max-i) > P[i'] (no expansion)
//Case 2: P[i]>=(max-i), iff (max-i) <= P[i'] (check for expansion)
//Case 3: P[i] = 0,      iff max-i < 0 
//
//That is, P[i] = max > i ? min(P[i'], max-i) : 0.

use crate::tprintln;
static DELIMETER: char = '$';

#[inline]
/// Symmetric property
fn mirror(of: usize, around: usize) -> usize {
    2 * around - of
}

struct PalindromeSearch { }

fn palindrome_sizes(chars: &[char]) -> Vec<usize> {
    let mut string_proccessed_to = 0;
    let mut current_palindrom_center = 0;
    let mut sizes = vec![0; chars.len()];
    for i in 0 .. sizes.len() {
        // Notation:
        // Main palindrome - palindrome which center defined by `current_palindrom_center`.
        // Right palindrome - palindrome considered at the current iteration. It's to the right
        // of `current_palindrom_center`.
        // Left palindrome - palindrome which center as far as center of Right palindrome but it's
        // to the left side.
        //
        // Size of Right palindrome equals to size of the Left palindrome if the distance from the
        // center of the ith palindrome (sizes[i]) to the right boundary is less than size of the
        // Left palindrome.
        //
        // We aren't sure what's after the right boundary but we know what's before. At this
        // point we know what's before because we move the right boundary to the last letter on
        // right side equals to its mirror (letter on the left side at the same distance from
        // the center of Right palindrome).
        sizes[i] = if i < string_proccessed_to {
            sizes[mirror(i, current_palindrom_center)].min(string_proccessed_to - i)
        } else {
            0
        };

        // size of Right palindrome from increments as letters on left side equal letters on
        // right side.
        while i + sizes[i] + 1 < chars.len() && i >= sizes[i] + 1
            && chars[i - sizes[i] - 1] == chars[i + sizes[i] + 1]
        {
            sizes[i] += 1;
        }

        // We shift right boundary by size of the Right palindrome because checked all letters
        // behind the boundary. The condition checks whether they were behind.
        if i + sizes[i] > string_proccessed_to {
            current_palindrom_center = i;
            string_proccessed_to = i + sizes[i];
        }
    }

    sizes
}

impl PalindromeSearch {
    fn new<T: AsRef<str>>(input: &T) -> String {
        let input: Vec<_> = input.as_ref().chars().collect();
        let mut chars = Vec::with_capacity(2 * input.len() + 1);
        chars.push(DELIMETER);
        input.into_iter().for_each(|c| {
            chars.push(c);
            chars.push(DELIMETER);
        });

        let palindrome_lengths = palindrome_sizes(&chars);
        let (pos, longest_palindrome_center) = palindrome_lengths.iter().enumerate()
            .fold((0, 0), |(i, max), (j, b)| if max < *b {
                (j, *b)
            } else {
                (i, max)
            });

        chars[pos - longest_palindrome_center .. pos + longest_palindrome_center + 1]
            .into_iter().filter(|x| **x != DELIMETER).collect()
    }
}

mod tests {
    use super::*;

    #[test]
    fn whole_word_is_palindrome() {
        assert_eq!(PalindromeSearch::new(&"abba"), "abba");
        assert_eq!(PalindromeSearch::new(&"racecar"), "racecar");
    }

    #[test]
    fn oneletter() {
        let d = DELIMETER;
        assert_eq!(palindrome_sizes(&[d, 'N', d]), vec![0, 1, 0]);
    }

    #[test]
    fn right_part_is_palindrome() {
        let d = DELIMETER;
        // aaabnbnb
        let input = [d, 'a', d, 'a', d, 'a', d, 'b', d, 'n', d, 'b', d, 'n', d, 'b', d];
        assert_eq!(palindrome_sizes(&input), vec![0, 1, 2, 3, 2, 1, 0, 1, 0, 3, 0, 5, 0, 3, 0, 1, 0]);
    }

    #[test]
    fn middle_part_is_palindrome() {
        assert_eq!(PalindromeSearch::new(&"cabal"), "aba");
    }

    #[test]

    #[test]
    fn lorem_ipsum() {
        let lorem_ipsum = include_str!("lorem_ipsum.txt");
        assert_eq!(PalindromeSearch::new(&lorem_ipsum), "is si");
    }
}
