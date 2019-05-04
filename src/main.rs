#![allow(unused)]
extern crate libc;
// static input: str = "EASY****";

use std::fmt::Debug;
mod list;
mod user;
mod process;
mod utils;
mod tree;

use std::marker::PhantomData;
use std::env::args;
use list::List;
use user::getpwnam;
use tree::Node;
use tree::make_tree;

fn main() {
}
