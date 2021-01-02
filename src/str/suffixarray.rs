struct SuffixArray<'a> {
    data: &'a str,
    suffixes: Vec<(usize, usize)>,
}

impl<'a> SuffixArray<'a> {
    fn new(data: &'a str) -> Self {
        let mut suffixes = (0 .. data.len() - 1).into_iter().map(|x| (x, data.len())).collect();
        Self {
            suffixes,
            data,
        }
    }
}

fn sort(data: &str, suffixes: Vec<(usize, usize)>) {
}
