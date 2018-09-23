use std::any::TypeId;

macro_rules! call {
    // accept like
    // fn callable(self, a: i32) {
    //      print!("Hello");
    // }
    (fn $fname:ident (self, $($farg:ident: $farg_type:ty),*) $fbody:expr) => (
        call!(@internal fn $fname (self, $($farg: $farg_type),*) -> () {$fbody});
    );
    // accept like
    // fn callable(a: i32) {
    //      print!("Hello");
    // }
    (fn $fname:ident ($($farg:ident: $farg_type:ty),*) $fbody:expr) => (
        call!(@internal fn $fname (, $($farg: $farg_type),*) -> () {$fbody});
    );
    (@internal fn $fname:ident ($($self_arg:ident)*, $($farg:ident: $farg_type:ty),*) -> $fres:ty {$($fbody:expr)*}) => (
        fn $fname($($self_arg,),* $($farg: $farg_type),*) -> $fres {
            println!(stringify!($fname));
            $(
                println!(stringify!($self_arg));
            )*
            println!("123");
        }
    );
    // Decorator doubles result of definad function.
    (fn $fname:ident ($($farg:ident: $farg_type:ty),*) -> $fres:ty {$($fbody:tt)*}) => (
        fn $fname($($farg: $farg_type),*) -> $fres {
            let res = (|| {
                $($fbody)*
            })();
            return res * 4;
        }
    );
}

struct point(i32, i32);

impl point {
    call! {
        fn callee2(a: i32, b: i32) {
            println!("Hello, world!!!");
            println!("Hello, world!!!");
        }
    }
    call! {
        fn callee(self, a: i32, b: i32) {
            println!("Hello, world!!!");
            println!("Hello, world!!!");
        }
    }
    call! {
        fn callee3(a: i32, b: i32) -> i32 {
            if a == 1 {
                let c = a + b;
                c
            } else {
                let d = a * b;
                d
            }
        }
    }
}

fn main() {
    let p = point(0, 0);
    p.callee(1, 2);
    point::callee2(1, 2);
    print!("{}", point::callee3(2, 3));
}
