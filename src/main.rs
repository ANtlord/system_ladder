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

use std::time;
use std::env::args;
use std::process::exit;
use std::os::unix::fs::MetadataExt;

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

fn random() -> u32 {
    let v = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_nanos() as u64;
    let mut random = v as u32;
    random ^= random << 13;
    random ^= random >> 17;
    random ^= random << 5;
    random
}

fn main() {
    for i in 0 .. 100 {
        println!("{}", random());
    }
    // let filename = args().skip(1).next().or_exit("Point filename");
    // println!("{}", filename);
    // let filename = args().skip(1).next().expect("Point filename");
    // let metadata = fs::File::open(filename).and_then(|x| x.metadata()).expect("Can't read file");
    // println!("{:o}", metadata.mode())
}
