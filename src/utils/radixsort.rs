use std::mem;

const RADIX: usize = u8::MAX as usize;

fn radixsort(arr: &[u8]) -> Vec<u8> {
    let mut count = [0usize; RADIX + 1];
    let mut aux = vec![b'0'; arr.len()];
    arr.iter().for_each(|x| count[*x as usize + 1] += 1);
    (0 .. count.len() - 1).for_each(|i| count[i + 1] += count[i]);
    arr.iter().for_each(|byte| {
        let index = *byte as usize;
        aux[count[index]] = *byte;
        count[index] += 1;
    });

    aux
}

fn lsd<T: AsRef<[u8]> + AsMut<[u8]>>(arr: &mut [T]) {
    let word_len = arr[0].as_ref().len();
    let mut aux = vec![Vec::<u8>::new(); arr.len()];
    for i in (0 .. word_len).rev() {
        let mut count = [0usize; RADIX + 1];
        let column = arr.iter().map(|x| x.as_ref()[i]);
        column.for_each(|x| count[x as usize + 1] += 1);
        (0 .. count.len() - 1).for_each(|i| count[i + 1] += count[i]);

        arr.iter().for_each(|word| {
            let word = word.as_ref();
            let index = word[i] as usize;
            aux[count[index]] = word.to_vec();
            count[index] += 1;
        });

        (0 .. arr.len()).for_each(|i| arr[i].as_mut().clone_from_slice(aux[i].as_slice()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod radixsort {
        use super::*;

        #[test]
        fn basic() {
            assert_eq!(radixsort(b"edcba"), b"abcde");
        }
    }

    mod lcd {
        use super::*;

        #[test]
        fn basic() {
            let mut input = vec![
                b"John".to_vec(),
                b"Jack".to_vec(),
                b"Alex".to_vec(),
                b"Bell".to_vec(),
            ];

            lsd(&mut input);
            assert_eq!(input, vec![
                b"Alex".to_vec(),
                b"Bell".to_vec(),
                b"Jack".to_vec(),
                b"John".to_vec(),
            ]);
        }
    }

    mod find_cyclic_rotations {
        use super::*;

        fn fingerprint(from: &[u8]) -> Vec<u8> {
            let mut cyclic_rotations = vec![Vec::<u8>::new(); from.len()];
            let mut count = 0;
            (0 .. from.len()).for_each(|i| {
                cyclic_rotations[count].extend_from_slice(&from[i .. from.len()]);
                cyclic_rotations[count].extend_from_slice(&from[0 .. i]);
                count += 1;
            });

            debug_assert_eq!(cyclic_rotations.len(), from.len());
            debug_assert_eq!(cyclic_rotations[0].len(), from.len());

            lsd(&mut cyclic_rotations);
            cyclic_rotations.swap_remove(0)
        }

        #[test]
        fn basic() {
            let input_data = vec![
                b"algorithms".to_vec(),
                b"polynomial".to_vec(),
                b"sortsuffix".to_vec(),
                b"boyermoore".to_vec(),
                b"structures".to_vec(),
                b"minimumcut".to_vec(),
                b"suffixsort".to_vec(),
                b"stackstack".to_vec(),
                b"binaryheap".to_vec(),
                b"digraphdfs".to_vec(),
                b"stringsort".to_vec(),
                b"digraphbfs".to_vec(),
            ];

            let mut fingerprints: Vec<_> = input_data.iter().map(|x| fingerprint(x)).collect();
            lsd(&mut fingerprints);
            let mut res = false;
            (0 .. fingerprints.len() - 1).for_each(|i| {
                if fingerprints[i] == fingerprints[i + 1] {
                    res = true;
                }
            });

            assert!(res);
        }
    }
}
