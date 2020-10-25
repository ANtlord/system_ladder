macro_rules! tprint {
    ($($arg:tt)*) => { 
        if cfg!(test) {
            print!($($arg)*);
        }
    };
}
