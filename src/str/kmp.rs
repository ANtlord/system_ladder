/// Substring search in linear time with Knuth-Morris-Prath algorithm.
use std::borrow::Borrow;
use std::ops::Index;

struct Radix {
    data: [Option<usize>; 255],
    len: usize,
}

impl Radix {
    fn new<Byte: Borrow<u8>>(pattern: impl Iterator<Item=Byte>) -> Self {
        let mut data = [None; 255];
        let mut count = 0;
        pattern.map(|x| *x.borrow()).for_each(|byte| {
            if data[byte as usize].is_none() {
                data[byte as usize] = Some(count);
                count += 1;
            }
        });

        Self { data, len: count }
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl<Byte: Borrow<u8>> Index<Byte> for Radix {
    type Output = Option<usize>;

    fn index(&self, index: Byte) -> &Self::Output {
        &self.data[*index.borrow() as usize]
    }
}

type TransitionMap = Vec<usize>;

// DFA for Knuth-Morris-Prath algorithm.
//
// FIXME: support Unicode. Idea: use a RB-tree, B-Tree or a hashmap for that.
// https://github.com/BurntSushi/suffix - repo implements suffix tree for unicode
// https://www.aclweb.org/anthology/C96-2200.pdf Knuth-Morris-Prath algorithm for unicode on
// chineese.
// http://yuex.in/post/2017/06/kmp-beauty.html Knuth-Morris-Prath supports unicode replacing DFA by
// NFA.
// http://monge.univ-mlv.fr/~mac/Articles-PDF/CP-1991-jacm.pdf - article describes two-way search
// which is combination of Knuth-Morris-Prath and Boyer-Moore algorithm and used in the STD.
struct StateMachine {
    states: Vec<TransitionMap>,
    radix: Radix,
    transition_index: usize,
}

impl StateMachine {
    fn new(pattern: &str, index: Radix) -> Self {
        let mut states = vec![vec![0usize; index.len()]; pattern.len() + 1];
        let mut x = 0;
        states[0][index[pattern.as_bytes()[0]].unwrap()] = 1;
        for j in 1 .. pattern.len() {
            let byte = pattern.as_bytes()[j];
            let (left, mut right) = states.split_at_mut(j);
            right[0].copy_from_slice(&left[x]);
            let key = index[byte].unwrap();
            right[0][key] = j + 1;
            x = left[x][key];
        }

        Self {
            states,
            radix: index,
            transition_index: 0,
        }
    }

    fn next(&mut self, text_byte: u8) -> bool {
        if let Some(index) = self.radix[text_byte] {
            self.transition_index = self.states[self.transition_index][index];
            self.transition_index == self.states.len() - 1
        } else {
            self.transition_index = 0;
            false
        }
    }
}

fn build_state_machine(pattern: &str) -> StateMachine {
    let radix_index = Radix::new(pattern.bytes());
    StateMachine::new(pattern, radix_index)
}


struct Search {
}

impl Search {
    fn in_stream<Byte: Borrow<u8>>(text: impl Iterator<Item=Byte>, pattern: &str) -> Option<usize> {
        let mut dfa = build_state_machine(pattern);
        text.enumerate()
            .find(|(_, byte)| dfa.next(*byte.borrow()))
            .map(|(i, _)| i - pattern.len() + 1)
    }
}

mod tests {
    use super::*;

    mod radix {
        use super::*;

        #[test]
        fn basic() {
            "".contains("");
            let st = Radix::new("abac".bytes());
            assert_eq!(st[b'a'], Some(0));
            assert_eq!(st[b'b'], Some(1));
            assert_eq!(st[b'c'], Some(2));
        }
    }

    mod state_machine {
        use super::*;

        #[test]
        fn basic() {
            let pattern = "ABABAC";
            let radix = Radix::new(pattern.bytes());
            let dfa = StateMachine::new(pattern, radix);
            let radix = dfa.radix;
            // A
            assert_eq!(dfa.states[0][radix[b'A'].unwrap()], 1);
            assert_eq!(dfa.states[0][radix[b'B'].unwrap()], 0);
            assert_eq!(dfa.states[0][radix[b'C'].unwrap()], 0);

            // B
            assert_eq!(dfa.states[1][radix[b'A'].unwrap()], 1);
            assert_eq!(dfa.states[1][radix[b'B'].unwrap()], 2);
            assert_eq!(dfa.states[1][radix[b'C'].unwrap()], 0);

            // A
            assert_eq!(dfa.states[2][radix[b'A'].unwrap()], 3);
            assert_eq!(dfa.states[2][radix[b'B'].unwrap()], 0);
            assert_eq!(dfa.states[2][radix[b'C'].unwrap()], 0);

            // B
            assert_eq!(dfa.states[3][radix[b'A'].unwrap()], 1);
            assert_eq!(dfa.states[3][radix[b'B'].unwrap()], 4);
            assert_eq!(dfa.states[3][radix[b'C'].unwrap()], 0);

            // A
            assert_eq!(dfa.states[4][radix[b'A'].unwrap()], 5);
            assert_eq!(dfa.states[4][radix[b'B'].unwrap()], 0);
            assert_eq!(dfa.states[4][radix[b'C'].unwrap()], 0);

            // C
            assert_eq!(dfa.states[5][radix[b'A'].unwrap()], 1);
            assert_eq!(dfa.states[5][radix[b'B'].unwrap()], 4);
            assert_eq!(dfa.states[5][radix[b'C'].unwrap()], 6);
        }
    }

    mod search {
        use super::*;

        #[test]
        fn empty() {
            assert!(Search::in_stream("".bytes(), "some").is_none())
        }

        #[test]
        fn second_match() {
            let text = "to be or not to be that's is the question";
            let pattern = "to be that";
            let res = Search::in_stream(text.bytes(), pattern);
            assert_eq!(res, Some(13));
            assert_eq!(&text[13 .. 13 + pattern.len()], pattern);
        }
    }

    #[test]
    fn cyclic_rotation() {
        let text = "winterbreak";
        let pattern = "breakwinter";

        let mut chunks = Vec::with_capacity(text.len() * 2);
        for i in 0 .. text.len() {
            chunks.push(&text[0 .. i]);
            chunks.push(&text[i .. text.len()]);
        }

        let letter_stream = chunks.iter().flat_map(|s| s.bytes());
        let res = Search::in_stream(letter_stream, pattern);
        assert!(res.is_some());
    }
}
