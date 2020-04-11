use std::os::raw::c_char;
use std::ffi;
use std::str;

pub fn from_cstr(v: *const c_char) -> Result<String, str::Utf8Error> {
    unsafe {
        ffi::CStr::from_ptr(v).to_str().map(String::from)
    }
}

