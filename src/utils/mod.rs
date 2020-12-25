pub mod vector;
mod quicksort;
mod insertionsort;
mod mergesort;
mod heapsort;
mod radixsort;

use std::mem;
use std::ptr;
use vector::sink;

pub use quicksort::quicksort;

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        mem::swap(&mut left, &mut right);
        right %= left;
    }
    left
}
