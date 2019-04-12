extern crate libc;
// static input: str = "EASY****";

mod list;
mod user;
mod process;

use std::env::args;

fn main() {
    for e in process::get_all_processes() {
        println!("{:#?}", e);
    }
}
