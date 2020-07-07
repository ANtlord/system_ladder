use core::mem;
use std::ffi::c_void;
use crate::utils::signal as sigutils;

pub fn check_alarm() {
    unsafe fn alarm_handler(sig: i32, info: &sigutils::SigInfo, _: *const c_void) {
        println!("Time's up! sig = {}. Timer id = {}", sig, info.fields.timer.si_timerid);
    }

    unsafe {
        let mask = {
            let mut mask_uninit = mem::MaybeUninit::uninit();
            libc::sigemptyset(mask_uninit.as_mut_ptr());
            mask_uninit.assume_init()
        };
        let mut old_disposition = mem::MaybeUninit::uninit();
        let new_disposition = libc::sigaction {
            sa_flags: libc::SA_RESTART | libc::SA_SIGINFO,
            sa_mask: mask,
            sa_sigaction: alarm_handler as libc::sighandler_t,
            sa_restorer: None,
        };

        libc::sigaction(libc::SIGALRM, &new_disposition, old_disposition.as_mut_ptr());
        old_disposition.assume_init();

        libc::alarm(1);
        libc::pause();
    }
}
