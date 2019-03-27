// static input: str = "EASY****";

mod list;
use list::List;
use std::io;
use std::{
    cell::Cell,
    // ptr::NonNull,
    // ptr::null_mut,
    // mem::transmute,
};

fn main() {
    let l: List<u8> = List::new();
    // let mut item = Item{ inner: None };
    // let list1 = RefCell::new(List::new());
    // for i in 0u8..5 {
        // list1.borrow_mut().add(i);
    // }

    // let mut list1_mut = list1.borrow_mut();
    // let mut iter = list1_mut.iter();
    // // list1.borrow_mut().remove(&mut iter);

    // // iter.next();
    // let mut counter = 0;
    // loop {
        // match iter.next() {
            // Some(x) => {
                // println!("{}", x);

                // counter += 1;
                // if counter == 8 { break; }
            // }
            // None => break,
        // };
        // // list1_mut.remove(&mut iter);
    // }

    // while let Some(y) = {iter}.next() {
        // println!("{}", y);

        // counter += 1;
        // if counter == 8 { break; }

        // list1.remove(&mut iter);
    // }
}
