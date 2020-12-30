/// Find the longest palindromic substring with Manacher's algorithm in linear time.
// SOH delimeter like from FIX protocol.
static DELIMETER: char = 1 as char;

#[inline]
/// Symmetric property
fn mirror(of: usize, around: usize) -> usize {
    2 * around - of
}

/// str -> $s$t$r$. Size of output (2n + 1).
fn normalize(input: std::str::Chars) -> Vec<char> {
    let mut chars = vec![DELIMETER];
    input.for_each(|c| {
        chars.push(c);
        chars.push(DELIMETER);
    });

    chars
}

/// Computes vector of sizes of all palindromes in the given string. Each palindrome has center at
/// the position that its size designated at in the vector.
///
/// The input string must be normalized.
fn palindrome_sizes(chars: &[char]) -> Vec<usize> {
    let mut string_proccessed_to = 0;
    let mut current_palindrom_center = 0;
    let mut sizes = vec![0; chars.len()];
    for i in 0 .. sizes.len() {
        // Notation:
        // Main palindrome - palindrome which center defined by `current_palindrom_center`.
        // Right palindrome - palindrome is being processed at the current iteration. It's to the
        // right of `current_palindrom_center`.
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

struct PalindromeSearch {}

impl PalindromeSearch {
    fn new<T: AsRef<str>>(input: &T) -> String {
        let input = input.as_ref();
        if input.len() == 0 {
            return String::default();
        }

        let chars = normalize(input.chars());
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
        assert_eq!(palindrome_sizes(&normalize("abba".chars())), &[0, 1, 0, 1, 4, 1, 0, 1, 0]);
        assert_eq!(
            palindrome_sizes(&normalize("racecar".chars())),
            &[0, 1, 0, 1, 0, 1, 0, 7, 0, 1, 0, 1, 0, 1, 0],
        );
    }

    #[test]
    fn oneletter() {
        assert_eq!(palindrome_sizes(&normalize("N".chars())), vec![0, 1, 0]);
    }

    #[test]
    fn right_part_is_palindrome() {
        assert_eq!(
            palindrome_sizes(&normalize("aaabnbnb".chars())),
            vec![0, 1, 2, 3, 2, 1, 0, 1, 0, 3, 0, 5, 0, 3, 0, 1, 0],
        );
    }

    #[test]
    fn left_part_is_palindrome() {
        assert_eq!(
            palindrome_sizes(&normalize("bnbnbaaa".chars())),
            vec![0, 1, 0, 3, 0, 5, 0, 3, 0, 1, 0, 1, 2, 3, 2, 1, 0],
        );
    }

    #[test]
    fn middle_part_is_palindrome() {
        assert_eq!(palindrome_sizes(&normalize("nazak".chars())), &[0, 1, 0, 1, 0, 3, 0, 1, 0, 1, 0]);
    }

    #[test]
    fn lorem_ipsum() {
        let lorem_ipsum = include_str!("lorem_ipsum.txt");
        assert_eq!(PalindromeSearch::new(&lorem_ipsum), "is si");
    }
}
