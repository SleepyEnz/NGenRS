use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Utility function to convert C string to Rust string (safe wrapper)
pub fn cstr_to_rust(url: *const c_char) -> Option<&'static str> {
    unsafe { CStr::from_ptr(url) }.to_str().ok()
}

/// Utility function to convert Rust string to C string (transfers ownership)
pub fn rust_to_cstring(text: String) -> *mut c_char {
    CString::new(text).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" 
fn free_c_string(s: *mut c_char) {
    if s.is_null() { return; }
    unsafe {
        // Convert the raw pointer back to a CString and immediately drop it
        drop(CString::from_raw(s));
    }
}