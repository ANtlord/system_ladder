use std::{thread, io};
use std::os::unix::io as unixio;
use std::os::unix::io::FromRawFd;
use std::os::unix::process as unixprocess;
use std::fs;
use std::ffi;
use std::time;
use std::io::{Read, BufRead};
use std::io::Write;
use std::process;
use std::mem;
use std::os::raw::{c_char, c_int, c_void, c_long, c_short, c_uint};
use std::str;
use std::ptr;
use std::fmt;
use std::fmt::Formatter;
use std::error::Error;
use std::cmp::max;
use utils::signal as sigutils;
use utils::string;

static mut PIPE_FDS: [unixio::RawFd; 2] = [-1, -1];

unsafe fn advanced_handler(sig: c_int, si: *const libc::siginfo_t, ctx: *const c_void) {
    if si.is_null() {
        println!("si is null");
    }
    println!("advanced handler called: sig = {}, si_code = {}", sig, (*si).si_code);
}

unsafe fn global_handler(_: u32) {
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
            }
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
        libc::raise(libc::SIGUSR1);
        thread::sleep(time::Duration::from_secs(1));
    }
}

unsafe fn sigaction_handler(sig: u32) {
    println!("sigaction_handler called, {}", ffi::CStr::from_ptr(sigutils::strsignal(sig as i32)).to_str().unwrap());
}

pub fn err_string() -> Result<String, str::Utf8Error> {
    unsafe {
        let errloc = libc::__errno_location();
        string::from_cstr(libc::strerror(*errloc))
    }
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
    for i in libc::SIGHUP..libc::SIGXFSZ {
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
    let new_disposition = libc::sigaction {
        sa_flags: libc::SA_RESTART | libc::SA_SIGINFO,
        sa_mask: mask,
        sa_sigaction: advanced_handler as libc::sighandler_t,
        sa_restorer: None,
    };

    libc::sigaction(libc::SIGUSR2, &new_disposition, old_disposition.as_mut_ptr());
    old_disposition.assume_init();

    let mut blocked_signals = &mut get_blocked_signals().unwrap() as *mut _;
    if libc::sigprocmask(libc::SIG_BLOCK, ptr::null(), blocked_signals) == -1 {
        println!("fail to get blocked signals, code: {}. Text: {}", *errloc, err_string().unwrap());
    }

    println!("blocked signals {:?}", sigutils::sig_set_str(&*blocked_signals));
    let mut signals_to_block = blocked_signals;
    if libc::sigaddset(signals_to_block, libc::SIGUSR2) == -1 {
        println!("fail to block a signal, code: {}. Text: {}", *errloc, err_string().unwrap());
    }

    if libc::sigprocmask(libc::SIG_BLOCK, signals_to_block, blocked_signals) == -1 {
        println!("fail to block a signal, code: {}. Text: {}", *errloc, err_string().unwrap());
    }

    println!("blocked signals {:?}", sigutils::sig_set_str(&get_blocked_signals().unwrap()));
    let init_blocked_sigs = sig_set_u32(&get_blocked_signals().unwrap());
    println!("init_blocked_sigs {:0>32b}", init_blocked_sigs);

    let pending = get_pending_signals().unwrap();
    println!("pending signals {:?}", sigutils::sig_set_str(&pending));
    thread::sleep(time::Duration::from_secs(1));

    libc::raise(libc::SIGUSR2);
    let pending = get_pending_signals().unwrap();
    println!("pending signals {:?}", sigutils::sig_set_str(&pending));
    thread::sleep(time::Duration::from_secs(1));

    let mut signals_to_unblock = blocked_signals;
    println!("try to unblock SIGUSR2");
    if libc::sigaddset(signals_to_unblock, libc::SIGUSR2) == -1 {
        println!("fail to unblock a signal, code: {}. Text: {}", *errloc, err_string().unwrap());
    }

    if libc::sigprocmask(libc::SIG_UNBLOCK, signals_to_unblock, blocked_signals) == -1 {
        println!("fail to block a signal, code: {}. Text: {}", *errloc, err_string().unwrap());
    }

    libc::pause();
}

unsafe extern fn alternate_stack_handler(sig: u32) {
    println!("sigaction_handler called, {}", string::from_cstr(sigutils::strsignal(sig as i32)).unwrap());
    let a = 1u32;
    println!("stack address = {:p}", &a);
}

pub unsafe fn alternative_stack() -> Res<()> {
    let errloc = libc::__errno_location();
    let stack = libc::stack_t {
        ss_sp: Box::into_raw(Box::new([0u8; libc::SIGSTKSZ])) as *mut _,
        ss_flags: 0,
        ss_size: libc::SIGSTKSZ,
    };

    if libc::sigaltstack(&stack, ptr::null_mut()) == -1 {
        Err(format!(
            "fail to alternate stack for a signal handler, code: {}, Text: {}", *errloc, err_string()?
        ))?;
    }

    let mut mask_uninit = mem::MaybeUninit::uninit();
    libc::sigemptyset(mask_uninit.as_mut_ptr());
    let mut mask = mask_uninit.assume_init();
    let mut old_disposition = mem::MaybeUninit::uninit();
    let new_disposition = libc::sigaction {
        sa_flags: libc::SA_ONSTACK,
        sa_mask: mask,
        sa_sigaction: alternate_stack_handler as libc::sighandler_t,
        sa_restorer: None,
    };

    libc::sigaction(libc::SIGUSR2, &new_disposition, old_disposition.as_mut_ptr());
    old_disposition.assume_init();
    let a = 1u32;
    println!("stack address = {:p}", &a);
    libc::pause();
    Ok(())
}

pub unsafe fn wait_signal() -> Res<()> {
    let mut mask_uninit = mem::MaybeUninit::uninit();
    if libc::sigemptyset(mask_uninit.as_mut_ptr()) == -1 {
        Err("Fail initializing signal set")?;
    }

    let mut mask = mask_uninit.assume_init();
    if libc::sigaddset(&mut mask, libc::SIGUSR2) == -1 {
        Err("Fail adding SIGUSR2 to the signal set")?;
    }

    if libc::sigprocmask(libc::SIG_SETMASK, &mask, ptr::null_mut()) == -1 {
        Err("fail blocking signals")?;
    }

    let mut siginfo_uninit = mem::MaybeUninit::uninit();
    let signal_num = sigutils::sigwaitinfo(&mask, siginfo_uninit.as_mut_ptr());
    if signal_num == -1 {
        Err("Fail to wait a signal")?;
    }

    let siginfo = siginfo_uninit.assume_init();
    let siginfo_ext: sigutils::SigInfo = mem::transmute(siginfo);
    println!("si_num = {}, si_code = {}", &siginfo_ext.si_signo, &siginfo_ext.si_code);
    println!("signal {} is received from pid = {}", signal_num, &siginfo_ext.fields.kill.si_pid);
    // println!("signal {} is received from process with id {}", signal_num, &siginfo_ext.fields.kill.si_pid);
    Ok(())
}

unsafe fn child_signal_handler(sig: i32) {
    println!("I caught a signanl from a child: {}", sig);
}

pub unsafe fn listen_dead_child() {
    unsafe {
        let sighandler = libc::signal(libc::SIGCHLD, advanced_handler as libc::sighandler_t);
        assert_ne!(sighandler, libc::SIG_ERR, "fail to changle SIGCHLD disposition");
        match libc::fork() {
            0 => {
                println!("I'm a child and I'm done;");
                return;
            },
            -1 => println!("fail to born a child"),
            x => println!("I've got a child with id = {}", x),
        };
        libc::pause();
    }
}

pub unsafe fn check_grandparent() -> Res<()> {
    let grandparent_id = process::id();
    let max_levels = 2;

    println!("I'm a grandfather. My id = {}", grandparent_id);
    match libc::fork() {
        0 => (),
        -1 => println!("fail to born a child"),
        x => {
            println!("i've got a child with id = {}", x);
            thread::sleep(time::Duration::from_secs(3));
            println!("Grandparent waits parent");
            libc::wait(ptr::null_mut());
            thread::sleep(time::Duration::from_secs(10));
            println!("Grandparent dies");
            return Ok(());
        }
    };

    match libc::fork() {
        0 => (),
        -1 => println!("fail to born a child"),
        x => {
            println!("parent's gonna die with id = {}", process::id());
            return Ok(());
        }
    };

    thread::sleep(time::Duration::from_secs(1));
    println!(
        "Grandchild parent id after its parent become a zombile is = {}. It's grandparent = {}",
        unixprocess::parent_id(),
        grandparent_id,
    );
    thread::sleep(time::Duration::from_secs(6));
    println!(
        "Grandchild parent id after death of its parent is = {}. It's grandparent = {}",
        unixprocess::parent_id(),
        grandparent_id,
    );
    thread::sleep(time::Duration::from_secs(10));
    println!(
        "Grandchild parent id after death of its parent is = {}. It's grandparent = {}",
        unixprocess::parent_id(),
        grandparent_id,
    );
    Ok(())
}
