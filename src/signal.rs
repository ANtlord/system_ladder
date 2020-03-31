use std::thread;
use std::os::unix::io as unixio;
use std::os::unix::io::FromRawFd;
use std::fs as stdfs;
use std::ffi;
use std::time;
use std::io::Read;
use std::io::Write;

static mut PIPE_FDS: [unixio::RawFd; 2] = [-1, -1];

unsafe extern fn global_handler(_: u32) {
    println!("signal handler hello");
    let mut tx_pipe = stdfs::File::from_raw_fd(PIPE_FDS[1]);
    if let Err(x) = tx_pipe.write("1".as_bytes()) {
        println!("fail to send data to pipe {}", x);
    }
}

unsafe fn signal_handling() {
    let errloc = libc::__errno_location();
    if libc::pipe(PIPE_FDS.as_mut_ptr()) != 0 && *errloc != 0 {
        println!("fail to handle signal, code: {}. Text: {}", *errloc, ffi::CStr::from_ptr(libc::strerror(*errloc)).to_str().unwrap());
        return;
    }

    let rx_pipe = PIPE_FDS[0];
    thread::spawn(move || {
        let mut rx_pipe = stdfs::File::from_raw_fd(rx_pipe);
        let mut buf = [0u8; 1024];
        match rx_pipe.read(&mut buf) {
            Ok(_) => println!("rx_pipe is read success."),
            Err(_) => println!("rx_pipe is read fail."),
        }
    });

    libc::signal(libc::SIGUSR1, global_handler as libc::sighandler_t);
    if *errloc != 0 {
        println!("fail to handle signal, code: {}. Text: {}", *errloc, ffi::CStr::from_ptr(libc::strerror(*errloc)).to_str().unwrap());
        return;
    }

    loop {
        thread::sleep(time::Duration::from_secs(1));
    }
}

