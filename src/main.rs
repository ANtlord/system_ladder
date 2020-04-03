#![allow(unused)]
extern crate libc;
extern crate inotify;
extern crate inotify_sys;
// static input: str = "EASY****";

use std::fmt::{Debug, Formatter};
mod fs;
mod watch;
mod list;
mod process;
mod tree;
mod user;
mod utils;
mod signal;

use std::str;
use std::ffi;
use std::cell::Cell;
use std::marker;
use std::pin::Pin;
use std::env::args;
use std::os::unix::fs::MetadataExt;
use std::process::exit;
use std::path;
use std::thread;
use std::time;
use std::io::Read;
use std::io::Write;
use std::error::Error;
use std::net;
use std::io;
use inotify_sys::read;
use std::ops::Deref;
use std::fmt;

trait Wrap {
    fn wrap(self, v: &str) -> Self;
}

impl<T> Wrap for Result<T, Box<dyn Error>> {
    fn wrap(self, value: &str) -> Self {
        let value = value.to_owned();
        self.map_err(|x| format!("{}: {}", &value, x).into())
    }
}


trait Exit<T> {
    fn or_exit(self, msg: &str) -> T;
}

impl<T, E> Exit<T> for Result<T, E> {
    fn or_exit(self, msg: &str) -> T {
        if let Ok(x) = self {
            return x;
        }
        println!("{}", msg);
        exit(1);
    }
}

impl<T> Exit<T> for Option<T> {
    fn or_exit(self, msg: &str) -> T {
        if self.is_some() {
            return self.unwrap();
        }
        println!("{}", msg);
        exit(1);
    }
}

unsafe fn process_permissions() {
    let errloc = libc::__errno_location();
    let initial_real_user_id = libc::getuid();
    let initial_effective_user_id = libc::geteuid();
    println!(
        "real = {}, effective_user_id = {}",
        initial_real_user_id,
        initial_effective_user_id,
    );

    println!("seteuid to real");
    libc::seteuid(initial_real_user_id);
    if *errloc != 0 {
        println!("fail to set uid 1001, code: {}. Text: {}", *errloc, ffi::CStr::from_ptr(libc::strerror(*errloc)).to_str().unwrap());
        return;
    }

    let real_user_id = libc::getuid();
    let effective_user_id = libc::geteuid();
    println!("real = {}, effective_user_id = {}", real_user_id, effective_user_id);

    println!("setuid to effective");
    libc::setuid(initial_effective_user_id);
    if *errloc != 0 {
        println!("fail to set uid 1001, code: {}. Text: {}", *errloc, ffi::CStr::from_ptr(libc::strerror(*errloc)).to_str().unwrap());
        return;
    }

    let real_user_id = libc::getuid();
    let effective_user_id = libc::geteuid();
    println!("real = {}, effective_user_id = {}", real_user_id, effective_user_id);

    libc::setresuid(libc::getuid(), libc::geteuid(), real_user_id); 
    let (mut uid, mut euid, mut suid) = (0, 0, 0);
    libc::getresuid(&mut uid, &mut euid, &mut suid); 
    println!("real = {}, effective_user_id = {}, saved_user_id = {}", uid, euid, suid);
}
#[derive(Debug)]
enum ReadErr {
    ClientClosedConnection,
    IO(io::Error),
}

impl From<io::Error> for ReadErr {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}

impl fmt::Display for ReadErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ReadErr::ClientClosedConnection => write!(f, "Client has closed the connection"),
            ReadErr::IO(x) => std::fmt::Display::fmt(&x, f),
        }
    }
}

impl Error for ReadErr {}

fn handle_conn(stream: &mut net::TcpStream) -> Result<(), ReadErr> {
    let mut buf = [0u8; 1024];
    let size = match stream.read(&mut buf) {
        Ok(x) if x == 0 => Err(ReadErr::ClientClosedConnection)?,
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
        Err(e) => Err(e)?,
    };

    let read_buf = &buf[ .. size];
    match str::from_utf8(read_buf) {
        Ok(x) => println!("{}", x),
        _ => println!("{:?}", read_buf),
    }
    Ok(())
}

fn accept(listener: &net::TcpListener, streams: &mut Vec<Option<net::TcpStream>>) -> Result<(), Box<dyn Error>> {
    match listener.accept() {
        Ok((x, _)) => {
            x.set_nonblocking(true)?;
            streams.push(Some(x));
            println!("new connection");
            Ok(())
        },
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(()),
        Err(e) => Err(e)?,
    }
}

struct Loop {
    streams: Vec<Option<net::TcpStream>>,
    listener: net::TcpListener,
}

impl Loop {
    fn new() -> Result<Self, Box<dyn Error>> {
        let mut listener = net::TcpListener::bind("127.0.0.1:8053")?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            streams: vec!(),
            listener,
        })
    }

    fn sleep(&self) {
        thread::sleep(time::Duration::from_millis(16));
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>>{
        self.sleep();
        accept(&self.listener, &mut self.streams)?;
        for ref mut opt in &mut self.streams {
            let mut took = opt.take();
            match took {
                Some(mut x) => match handle_conn(&mut x) {
                    Ok(_) => {opt.replace(x);},
                    Err(e) => println!("connection lost. Reason: {}", e),
                },
                None => continue,
            };
        }
        Ok(())
    }
}

fn tcp_server() -> Result<(), Box<dyn Error>> {
    let mut evloop = Loop::new()?;
    loop {
        evloop.run()?;
    }
    Ok(())
}

fn main() {
    // tcp_server().unwrap();
    // unsafe {signal::signal_handling()};
    unsafe {signal::block_signals()};
}
