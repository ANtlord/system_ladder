#![allow(unused)]
extern crate libc;
// static input: str = "EASY****";

use std::fmt::Debug;
mod list;
mod user;
mod process;
mod utils;
mod tree;
mod fs;
mod container;
mod random;
mod graph;
mod trace;
mod str;
#[macro_use]
mod macros;

use std::time;
use std::env::args;
use std::process::exit;
use std::os::unix::fs::MetadataExt;
use crate::random::xorshift_rng as random;

trait Exit<T> {
    fn or_exit(self, msg: &str) -> T;
}

impl<T, E: Debug> Exit<T> for Result<T, E> {
    fn or_exit(self, msg: &str) -> T {
        if self.is_ok() {
            return self.unwrap();
        }
        println!("{}", msg);
        exit(1);
    }
}

impl<T> Exit<T> for Option<T> {
    fn or_exit(self, msg: &str) -> T {
        if self.is_some() {
            return self.unwrap();
        }
        println!("{}", msg);
        exit(1);
    }
}

fn main() {
    //let ret: Vec<u32> = vec![0; 3].into_iter().map(|_| random()).collect();
    //println!("{:?}", ret);
    // let filename = args().skip(1).next().or_exit("Point filename");
    // println!("{}", filename);
    // let filename = args().skip(1).next().expect("Point filename");
    // let metadata = fs::File::open(filename).and_then(|x| x.metadata()).expect("Can't read file");
    // println!("{:o}", metadata.mode())
}
