/// Moves elements have a field equals to sample.
///
/// # Examples
///
/// ```
/// let mut numbers = vec![0, 1, 1, 0, 3, 2, 0];
/// move_to_end_by(&0, &mut numbers, |x| &x);
/// let len = numbers.len();
/// assert_eq!(numbers[len - 1], 0);
/// assert_eq!(numbers[len - 2], 0);
/// assert_eq!(numbers[len - 3], 0);
/// ```
pub fn move_to_end_by<T, F>(sample: &T, heap: &mut [T], are_equal: F) -> Option<usize>
where
    F: Fn(&T, &T) -> bool,
{
    let heap_len = heap.len();
    match heap_len {
        1 if are_equal(&sample, &heap[0]) => Some(0),
        1 | 0 => None,
        _ => {
            let mut count = heap_len;

            for i in 0..count {

                if !are_equal(&sample, &heap[i]) {
                    continue;
                }

                while count == heap_len || are_equal(&sample, &heap[count]) && count > i {
                    count -= 1;
                }

                heap.swap(i, count);

            }

            Some(count)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_with_duplicates() {
        let mut numbers = vec![0, 1, 1, 0, 3, 2, 0];
        let since = move_to_end_by(&0, &mut numbers, |x, y| x == y);
        assert_eq!(since.unwrap(), 4);
    }

    #[test]
    fn sample_no_duplicates() {
        let mut numbers = vec![0, 1, 1, 0, 3, 2, 0];
        let since = move_to_end_by(&26, &mut numbers, |x, y| x == y);
        assert_eq!(since.unwrap(), numbers.len());
    }
}
