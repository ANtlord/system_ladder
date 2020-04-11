use std::mem;
use std::os::raw::{c_char, c_int, c_void, c_long, c_short, c_uint};
use utils::string;

const SI_MAX_SIZE: usize = 128;
const WORDSIZE: usize = 64;
const SI_PAD_SIZE: usize = ((SI_MAX_SIZE / mem::size_of::<i32>()) - 4);

extern "C" {
    pub fn psignal(sig: c_int, msg: *const c_char);
    pub fn strsignal(sig: c_int) -> *const c_char;
    pub fn sigwaitinfo(mask: *const libc::sigset_t, sig: *mut SigInfo) -> c_int;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Kill {
    pub si_pid: libc::pid_t,
    pub si_uid: libc::uid_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Timer {
    pub si_timerid: c_int,
    pub si_overrun: c_int,
}

#[repr(C)]
pub union SiFields {
    _pad: [i32; SI_PAD_SIZE],
    pub kill: Kill,
    pub timer: Timer,

    pub si_trapno: c_int,
    pub si_status: c_int,
    pub si_utime: libc::clock_t,
    pub si_stime: libc::clock_t,
    pub si_value: c_int,
    pub si_int: c_int,
    pub si_ptr: *const c_void,
    pub si_addr: *const c_void,
    pub si_band: c_long,
    pub si_fd: c_int,
    pub si_addr_lsb: c_short,
    pub si_lower: *const c_void,
    pub si_upper: *const c_void,
    pub si_pkey: c_int,
    pub si_call_addr: *const c_void,
    pub si_syscall: c_int,
    pub si_arch: c_uint,
}

#[repr(C)]
pub struct SigInfo {
    pub si_signo: c_int,
    pub si_errno: c_int,
    pub si_code: c_int,
    pub fields: SiFields,
}

pub unsafe fn sig_set_str(val: &libc::sigset_t) -> String {
    (libc::SIGHUP .. libc::SIGXFSZ + 1).filter(|x| libc::sigismember(val, *x) == 1)
        .map(|x| string::from_cstr(strsignal(x)).unwrap())
        .collect::<Vec<String>>().join(", ")
}
