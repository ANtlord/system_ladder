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

static DELIMETER: char = 0x01 as char;

static LOREM_IPSUM: &str = r#"
Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the
industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type
and scrambled it to make a type specimen book. It has survived not only five centuries, but also
the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the
1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with
desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.
"#;


#[inline]
/// Symmetric property
fn mirror(of: usize, around: usize) -> usize {
    2 * around - of
}

struct PalindromeSearch { }

impl PalindromeSearch {
    fn new<T: AsRef<str>>(input: &T) -> String {
        let input: Vec<_> = input.as_ref().chars().collect();
        let mut chars = Vec::with_capacity(2 * input.len() + 1);
        chars.push(DELIMETER);
        input.iter().for_each(|c| {
            chars.push(*c);
            chars.push(DELIMETER);
        });

        let mut max = 0;
        let mut current_palindrom_center = 0;
        let mut palindrome_lengths = vec![0; chars.len()];
        for i in 0 .. palindrome_lengths.len() {
            palindrome_lengths[i] = if max > i {
                palindrome_lengths[mirror(i, current_palindrom_center)].min(max - i)
            } else {
                0
            };

            while i + palindrome_lengths[i] + 1 < chars.len()
                && i > palindrome_lengths[i] + 1
                && chars[i - palindrome_lengths[i] - 1] == chars[i + palindrome_lengths[i] + 1] {
                palindrome_lengths[i] += 1;
            }

            if i + palindrome_lengths[i] > max {
                current_palindrom_center = i;
                max = i + palindrome_lengths[i];
            } else {
                dbg!(max, i, palindrome_lengths[i]);
            }
        }

        let (pos, longest_palindrome_center) = palindrome_lengths.iter().enumerate()
            .fold((0, 0), |(i, a), (j, b)| if a < *b {
                (j, *b)
            } else {
                (i, a)
            });

        chars[pos - longest_palindrome_center .. pos + longest_palindrome_center + 1]
            .into_iter().filter(|x| **x != DELIMETER).collect()
    }
}

mod tests {
    use super::*;

    #[test]
    fn abba() {
        assert_eq!(PalindromeSearch::new(&"abba"), "abba");
        assert!(false);
    }

    #[test]
    fn racecar() {
        assert_eq!(PalindromeSearch::new(&"racecar"), "racecar");
    }

    #[test]
    fn cabal() {
        assert_eq!(PalindromeSearch::new(&"cabal"), "aba");
    }

    #[test]
    fn lorem_ipsum() {
        assert_eq!(PalindromeSearch::new(&LOREM_IPSUM), "is si");
    }
}
