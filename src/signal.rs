use std::{thread, io};
use std::os::unix::io as unixio;
use std::os::unix::io::FromRawFd;
use std::fs;
use std::ffi;
use std::time;
use std::io::Read;
use std::io::Write;
use std::process;
use std::mem;
use std::os::raw::{c_char, c_int};
use std::str;
use std::ptr;
use std::fmt;
use std::fmt::{Formatter};
use std::error::Error;

static mut PIPE_FDS: [unixio::RawFd; 2] = [-1, -1];

unsafe extern fn global_handler(_: u32) {
    println!("signal handler hello");
    let mut tx_pipe = fs::File::from_raw_fd(PIPE_FDS[1]);
    if let Err(x) = tx_pipe.write("1".as_bytes()) {
        println!("fail to send data to pipe {}", x);
    }
}

unsafe fn read_pipe(rx_pipe: i32) {
    println!("try to read");
    let mut pipe_fd: fs::File = fs::File::from_raw_fd(rx_pipe);
    let mut buf = [0u8; 1024];
    loop {
        thread::sleep(time::Duration::from_secs(2));
        match pipe_fd.read(&mut buf) {
            Ok(s) if s == 0 => break,
            Ok(s) => {
                println!("rx_pipe is read success.");
                process::exit(1);
            },
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Err(e) => println!("rx_pipe is read fail. Text: {}", e),
        }
    }
    println!("qweasd");
}

pub unsafe fn signal_handling() {
    let errloc = libc::__errno_location();
    if libc::pipe(PIPE_FDS.as_mut_ptr()) != 0 && *errloc != 0 {
        println!("fail to handle signal, code: {}. Text: {}", *errloc, err_string().unwrap());
        return;
    }

    let rx_pipe = PIPE_FDS[0];
    let pipe_flags = libc::fcntl(rx_pipe, libc::F_GETFL);
    if pipe_flags == -1 {
        println!("fail to get pipe flags, code: {}. Text: {}", *errloc, err_string().unwrap());
        return;
    }

    if libc::fcntl(rx_pipe, libc::F_SETFL, pipe_flags | libc::O_NONBLOCK) == -1 {
        println!("fail to set pipe flags, code: {}. Text: {}", *errloc, err_string().unwrap());
        return;
    }

    thread::spawn(move || read_pipe(rx_pipe));

    libc::signal(libc::SIGUSR1, global_handler as libc::sighandler_t);
    if *errloc != 0 {
        println!("fail to handle signal, code: {}. Text: {}", *errloc, err_string().unwrap());
        return;
    }

    loop {
        thread::sleep(time::Duration::from_secs(1));
    }
}

unsafe extern fn sigaction_handler(sig: u32) {
    println!("sigaction_handler called, {}", ffi::CStr::from_ptr(strsignal(sig as i32)).to_str().unwrap());
}

extern "C" {
    fn psignal(sig: c_int, msg: *const c_char);
    fn strsignal(sig: c_int) -> *const c_char;
}

pub fn err_string() -> Result<String, str::Utf8Error> {
    unsafe {
        let errloc = libc::__errno_location();
        ffi::CStr::from_ptr(libc::strerror(*errloc)).to_str().map(String::from)
    }
}

pub fn cstr_str(v: *const c_char) -> Result<String, str::Utf8Error> {
    unsafe {
        ffi::CStr::from_ptr(v).to_str().map(String::from)
    }
}


unsafe fn sig_set_str(val: &libc::sigset_t) -> String {
    (libc::SIGHUP .. libc::SIGXFSZ).filter(|x| libc::sigismember(val, *x) == 1)
        .map(|x| cstr_str(strsignal(x)).unwrap())
        .collect::<Vec<String>>().join(", ")
}

type Res<T> = Result<T, Box<dyn Error>>;

unsafe fn get_blocked_signals() -> Res<libc::sigset_t> {
    let mut blocked_signals_uninit = mem::MaybeUninit::uninit();
    if libc::sigprocmask(libc::SIG_BLOCK, ptr::null(), blocked_signals_uninit.as_mut_ptr()) == -1 {
        Err(format!(
            "fail to get blocked signals, code: {}. Text: {}",
            *libc::__errno_location(),
            err_string().unwrap()),
        )?
    }

    Ok(blocked_signals_uninit.assume_init())
}

unsafe fn get_pending_signals() -> Res<libc::sigset_t> {
    let mut pending_signals_uninit = mem::MaybeUninit::uninit();
    if libc::sigpending(pending_signals_uninit.as_mut_ptr()) == -1 {
        Err(format!(
            "fail to get pending signals, code: {}. Text: {}",
            *libc::__errno_location(),
            err_string().unwrap()),
        )?
    }

    Ok(pending_signals_uninit.assume_init())
}

unsafe fn sig_set_u32(val: &libc::sigset_t) -> u32 {
    let mut res = 0;
    for i in libc::SIGHUP .. libc::SIGXFSZ {
        if libc::sigismember(val, i) == 1 {
            res |= 1;
        }
        res <<= 1;
    }
    res
}

pub unsafe fn block_signals() {
    let errloc = libc::__errno_location();
    let mut mask_uninit = mem::MaybeUninit::uninit();
    libc::sigemptyset(mask_uninit.as_mut_ptr());
    let mut mask = mask_uninit.assume_init();
    let mut old_disposition = mem::MaybeUninit::uninit();
    let new_disposition = libc::sigaction{
        sa_flags: libc::SA_RESTART,
        sa_mask: mask,
        sa_sigaction: sigaction_handler as libc::sighandler_t,
        sa_restorer: None,
    };

    libc::sigaction(libc::SIGUSR2, &new_disposition, old_disposition.as_mut_ptr());
    old_disposition.assume_init();

    let mut blocked_signals = &mut get_blocked_signals().unwrap() as *mut _;
    if libc::sigprocmask(libc::SIG_BLOCK, ptr::null(), blocked_signals) == -1 {
        println!("fail to get blocked signals, code: {}. Text: {}", *errloc, err_string().unwrap());
    }

    println!("blocked signals {:?}", sig_set_str(&*blocked_signals));
    let mut signals_to_block = blocked_signals;
    if libc::sigaddset(signals_to_block, libc::SIGUSR2) == -1 {
        println!("fail to block a signal, code: {}. Text: {}", *errloc, err_string().unwrap());
    }

    if libc::sigprocmask(libc::SIG_BLOCK, signals_to_block, blocked_signals) == -1 {
        println!("fail to block a signal, code: {}. Text: {}", *errloc, err_string().unwrap());
    }

    println!("blocked signals {:?}", sig_set_str(&get_blocked_signals().unwrap()));
    let init_blocked_sigs = sig_set_u32(&get_blocked_signals().unwrap());
    println!("init_blocked_sigs {:0>32b}", init_blocked_sigs);

    let pending = get_pending_signals().unwrap();
    println!("pending signals {:?}", sig_set_str(&pending));
    thread::sleep(time::Duration::from_secs(1));

    libc::raise(libc::SIGUSR2);
    let pending = get_pending_signals().unwrap();
    println!("pending signals {:?}", sig_set_str(&pending));
    thread::sleep(time::Duration::from_secs(1));

    let mut signals_to_unblock = blocked_signals;
    println!("try to unblock SIGUSR2");
    if libc::sigaddset(signals_to_unblock, libc::SIGUSR2) == -1 {
        println!("fail to unblock a signal, code: {}. Text: {}", *errloc, err_string().unwrap());
    }

    if libc::sigprocmask(libc::SIG_UNBLOCK, signals_to_unblock, blocked_signals) == -1 {
        println!("fail to block a signal, code: {}. Text: {}", *errloc, err_string().unwrap());
    }
}
