use std::time;

/// Pseudorandom number generator from the "Xorshift RNGs" paper by George Marsaglia.
pub fn xorshift_rng() -> u32 {
    let v = time::SystemTime::now().duration_since(time::UNIX_EPOCH)
        .unwrap().as_nanos() as u32;
    let mut random = v;
    random ^= random << 13;
    random ^= random >> 17;
    random << 5
}
