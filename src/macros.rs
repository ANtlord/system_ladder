#[macro_export]
macro_rules! tprintln {
    ($($arg:tt)*) => { 
        if cfg!(test) {
            println!($($arg)*);
        }
    };
}
