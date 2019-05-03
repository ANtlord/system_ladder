#![allow(unused)]
extern crate libc;
// static input: str = "EASY****";

use std::fmt::Debug;
mod list;
mod user;
mod process;
mod utils;
mod tree;

use std::env::args;
use list::List;
use user::getpwnam;
use tree::Node;
use tree::make_tree;
//use std::collections;

fn main() {
    //let user_data = args().skip(1).next().and_then(|x| getpwnam(&x).ok()).expect("Provide user name");

    let mut user_process_vec = vec!();
    for e in process::get_all_processes() {
        user_process_vec.push(e);
    }

    let mut fake_process = Node::new(process::Process{name: "".to_owned(), pid: 0, ppid: 0});
    make_tree(&mut fake_process, &mut user_process_vec, |x, y| x.pid == y.ppid);
    debug_assert_eq!(user_process_vec.len(), 0, "{:?}", &user_process_vec);
    print_tree(&fake_process, 0);
}

fn print_tree(n: &Node<process::Process>, indent: usize) {
    println!("{}> {} {} {}", "-".repeat(indent), n.value.name, n.value.pid, n.value.ppid);
    for e in &n.children {
        print_tree(&e, indent + 1)
    }
}
