#![allow(unused)]
extern crate libc;
// static input: str = "EASY****";

mod list;
mod user;
mod process;
mod utils;

use std::env::args;
use list::List;
use user::getpwnam;

fn main() {
    let user_data = args().skip(1).next().and_then(|x| getpwnam(&x).ok()).expect("Provide user name");

    let mut user_process_vec = vec!();
    for e in process::get_processes(user_data.user_id) {
        user_process_vec.push(e);
    }

    utils::quicksort(&mut user_process_vec, |x, y| x.ppid > y.ppid);
}
