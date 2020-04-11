pub mod vector;
pub mod sort;
pub mod signal;
pub mod string;

use std::ffi;
use std::mem;

/// Return greatest common divisor
fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        mem::swap(&mut left, &mut right);
        right %= left;
    }
    left
}
