#![allow(unused)]
extern crate libc;
// static input: str = "EASY****";

mod list;
mod user;
mod process;

use std::env::args;
use list::List;
use user::getpwnam;
use std::mem;

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        mem::swap(&mut left, &mut right);
        right %= left;
    }
    return left;
}

fn quicksort<T: Ord>(array: &mut [T]) {
    println!("quicksort");
    let len = array.len();

    if len <= 1 {
        return;
    }

    let last = len - 1;
    let random_pivot_position = len / 2;
    array.swap(random_pivot_position, last);

    let mut i = 0;
    let mut j = last - 1;
    {
        while i <= j {
            if &array[i] < &array[last] {
                i += 1;
                continue;
            }

            if &array[j] > &array[last] {
                j -= 1;
                continue;
            }

            array.swap(i, j);
        }
    }
    array.swap(i, last);

    quicksort(&mut array[0 .. i]);
    quicksort(&mut array[i .. len]);
}

fn main() {
    //let user_data = args().skip(1).next().and_then(|x| getpwnam(&x).ok()).expect("Provide user name");

    //let mut user_process_vec = vec!();
    //for e in process::get_processes(user_data.user_id) {
    //    user_process_vec.push_back(e);
    //}
    //user_process_vec.sort_by(|(x, y)| x.ppid > y.ppid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odd_elements() {
        let mut v = vec![3,2,1];
        quicksort(&mut v);
        assert_eq!(v, vec![1,2,3]);
    }

    #[test]
    fn even_elements() {
        let mut v = vec![4,3,2,1];
        quicksort(&mut v);
        assert_eq!(v, vec![1,2,3,4]);
    }

}
