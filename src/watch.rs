use std::thread;
use std::time;
use std::pin::Pin;
use std::future::Future;
use std::marker::PhantomPinned;
use std::ptr::NonNull;
use std::task::{Context, Poll};
use std::ffi::{CString, CStr};
use std::ffi;
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::io::Read;
use std::os::unix::io::FromRawFd;
use std::fs;
use std::str::Utf8Error;
use std::error::Error as StdErr;
use std::mem;
use std::path;
use std::os::unix::ffi::OsStrExt;
use std::env;
use inotify::{
    EventMask,
    WatchMask,
    Inotify,
};

fn call(callable: fn(i32) -> NumberFut) {
}

extern "C" {
    fn inotify_init() -> c_int;
    fn inotify_init1(flags: c_int) -> c_int;
    fn inotify_add_watch(fd: c_int, pathname: *const c_char, mask: c_uint) -> c_int;
}

#[derive(Debug)]
struct InotifyEvent {
   wd:    c_int  ,
   mask:  c_uint ,
   cookie:c_uint ,
   len:   c_uint ,
}

const BUF_SIZE: usize  = 1024 * (mem::size_of::<InotifyEvent>() + 16);

unsafe fn run_watch(wd: i32) {
    let mut flag = true;
    // let mut wd_file = fs::File::from_raw_fd(wd as c_int);
    let mut buf = [0u8; BUF_SIZE];
    println!("buf pointer is {:p}", buf.as_ptr());
    let ptr_val = buf.as_ptr() as u64;
    println!("buf pointer is number {:x}", ptr_val);
    while flag {
        // println!("looking for new IO events. wd is {}", wd);
        thread::sleep(time::Duration::from_millis(16));
        let res = libc::read(wd, buf.as_mut_ptr() as *mut c_void, buf.len());
        if res < 0 {
            continue;
        }

        println!("read of watch descriptor happened");
        if (res as usize) < mem::size_of::<InotifyEvent>() {
            println!("unexpected size of the pointer: {}", res);
            continue;
        }

        let ev_ptr = buf.as_ptr() as *const InotifyEvent;
        let ev: &InotifyEvent = unsafe { &*ev_ptr };
        println!("event mask is {:?}", ev);
        
        // let res = wd_file.read(&mut buf);
        // match res {
        //     Ok(x) => println!("read {}", x),
        //     Err(e) => {
        //         println!("fail read, {}", e);
        //         flag = false;
        //     },
        // }
    }
}

unsafe fn get_err(errno: c_int) -> Result<String, Box<dyn StdErr>> {
    let error = libc::strerror(errno);
    if error.is_null() {
        panic!("Fail getting error text");
    }
    let cstr = CStr::from_ptr(error);
    Ok(String::from(cstr.to_str()?))
}

pub fn main_w() {
    let mut inotify = Inotify::init()
        .expect("Failed to initialize inotify");

    let current_dir = env::current_dir()
        .expect("Failed to determine current directory");

    inotify
        .add_watch(
            "/home/wantlord/develop/system_ladder/qwe",
            WatchMask::MODIFY,
        )
        .expect("Failed to add inotify watch");

    println!("Watching current directory for activity...");

    let mut buffer = [0u8; 4096];
    loop {
        let events = inotify
            .read_events_blocking(&mut buffer)
            .expect("Failed to read inotify events");

        println!("event happened");
        // for event in events {
        //     if event.mask.contains(EventMask::CREATE) {
        //         if event.mask.contains(EventMask::ISDIR) {
        //             println!("Directory created: {:?}", event.name);
        //         } else {
        //             println!("File created: {:?}", event.name);
        //         }
        //     } else if event.mask.contains(EventMask::DELETE) {
        //         if event.mask.contains(EventMask::ISDIR) {
        //             println!("Directory deleted: {:?}", event.name);
        //         } else {
        //             println!("File deleted: {:?}", event.name);
        //         }
        //     } else if event.mask.contains(EventMask::MODIFY) {
        //         if event.mask.contains(EventMask::ISDIR) {
        //             println!("Directory modified: {:?}", event.name);
        //         } else {
        //             println!("File modified: {:?}", event.name);
        //         }
        //     }
        // }
    }
}

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}
 
pub fn main() {
    let start = Point{ x: 123., y: 412. };
    let finish = Point{ y: 814., ..start };
    println!("finish point is {:?}", finish);

    let path = path::Path::new("/home/wantlord/develop/system_ladder/qwe");
    let mask = inotify_sys::IN_MODIFY;
    let tmp = CString::new(path.as_os_str().as_bytes()).unwrap();
    unsafe {
        let mut errno = libc::__errno_location();
        if errno.is_null() {
            panic!("errno is null")
        }

        let inotify_init_fd = inotify_init1(inotify_sys::IN_CLOEXEC | inotify_sys::IN_NONBLOCK);
        match inotify_init_fd {
            -1 => panic!("fail inotify_init"),
            _ => println!("inotify_init success: {}", inotify_init_fd),
        }

        let tmp_watch_fd = inotify_add_watch(inotify_init_fd, tmp.as_ptr() as *const _, mask);
        if tmp_watch_fd == -1 {
            match *errno {
                0 => panic!("fail inotify_watch"),
                _ => panic!("fail inotify_watch. {} - {}", get_err(*errno).unwrap(), *errno),
            }
        }

        println!("inotify_add_watch success: {}", inotify_init_fd);
        run_watch(inotify_init_fd);
    }
    call(NumberFut);
}

struct NumberFut(i32);
