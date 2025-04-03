use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use libc;
use std::slice;

/// Utility function to convert C string to Rust string (safe wrapper)
pub fn cstr_to_rust(cstr: *const c_char) -> Option<&'static str> {
    if cstr.is_null() {
        return None;
    }
    unsafe {
        let len = libc::strlen(cstr);
        if len > isize::MAX as usize {
            return None;
        }
        CStr::from_ptr(cstr).to_str().ok()
    }
}

/// Utility function to convert Rust string to C string (transfers ownership)
pub fn rust_to_cstr(rstr: String) -> *mut c_char {
    match CString::new(rstr) {
        Ok(cstr) => cstr.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Converts C byte array to Rust slice (safe wrapper)
pub fn cbytes_to_rust(data: *const u8, len: usize) -> Option<&'static [u8]> {
    if data.is_null() {
        None
    } else {
        unsafe {
            if len <= isize::MAX as usize {
                Some(slice::from_raw_parts(data, len))
            } else {
                None
            }
        }
    }
}

/// Converts Rust Vec<u8> to C-owned bytes (transfers ownership)
pub fn rust_to_cbytes(data: Vec<u8>) -> (*mut u8, usize) {
    let boxed = data.into_boxed_slice();
    let len = boxed.len();
    (Box::into_raw(boxed) as *mut u8, len)
}

pub fn free<T>(_x: T) {
    drop(_x);
}

pub fn ngenrs_free_ptr<T>(raw: *mut T) {
    if !raw.is_null() {
        unsafe { free(Box::from_raw(raw)) };
    }
}

// Replace the manually written exports with macro versions
#[unsafe(no_mangle)]
pub extern "C"
fn ngenrs_free_cstr(s: *mut c_char) {
    free(unsafe { CString::from_raw(s) });
}

#[unsafe(no_mangle)]
pub extern "C"
fn ngenrs_free_bytes(buf: *mut u8, len: usize) {
    ngenrs_free_ptr(unsafe { slice::from_raw_parts_mut(buf, len).as_mut_ptr() });
}

pub fn box_into_raw_new<T>(value: T) -> *mut T {
    Box::into_raw(Box::new(value))
}