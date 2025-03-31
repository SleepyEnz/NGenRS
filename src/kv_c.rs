use std::os::raw::c_char;
use crate::kv::KV;
use crate::cc::{cstr_to_rust, rust_to_cstring};

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_kv_open(path: *const c_char) -> *mut KV {
    let path_str = match cstr_to_rust(path) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    
    match KV::open(path_str) {
        Ok(store) => Box::into_raw(Box::new(store)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_kv_write_int(store: *mut KV, key: *const c_char, value: i64) -> bool {
    if store.is_null() { return false; }
    let key_str = match cstr_to_rust(key) {
        Some(s) => s,
        None => return false,
    };
    unsafe {
        let kv_ref = &mut *store;
        match kv_ref.write_int(key_str, value) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_kv_read_int(store: *mut KV, key: *const c_char) -> i64 {
    if store.is_null() { return 0; }
    let key_str = match cstr_to_rust(key) {
        Some(s) => s,
        None => return 0,
    };
    unsafe {
        let kv_ref = &mut *store;
        match kv_ref.read_int(key_str) {
            Ok(value) => value.unwrap_or(0),
            Err(_) => 0,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_kv_write_float(store: *mut KV, key: *const c_char, value: f64) -> bool {
    if store.is_null() { return false; }
    let key_str = match cstr_to_rust(key) {
        Some(s) => s,
        None => return false,
    };
    unsafe {
        let kv_ref = &mut *store;
        match kv_ref.write_float(key_str, value) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_kv_read_float(store: *mut KV, key: *const c_char) -> f64 {
    if store.is_null() { return 0.0; }
    let key_str = match cstr_to_rust(key) {
        Some(s) => s,
        None => return 0.0,
    };
    unsafe {
        let kv_ref = &mut *store;
        match kv_ref.read_float(key_str) {
            Ok(value) => value.unwrap_or(0.0),
            Err(_) => 0.0,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_kv_write_string(store: *mut KV, key: *const c_char, value: *const c_char) -> bool {
    if store.is_null() { return false; }
    let key_str = match cstr_to_rust(key) {
        Some(s) => s,
        None => return false,
    };
    let value_str = match cstr_to_rust(value) {
        Some(s) => s,
        None => "",
    };
    unsafe {
        let kv_ref = &mut *store;
        match kv_ref.write_string(key_str, value_str) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_kv_read_string(store: *mut KV, key: *const c_char) -> *mut c_char {
    if store.is_null() { return std::ptr::null_mut(); }
    let key_str = match cstr_to_rust(key) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    unsafe {
        let kv_ref = &mut *store;
        match kv_ref.read_string(key_str) {
            Ok(value) => match value {
                Some(s) => rust_to_cstring(s),
                None => std::ptr::null_mut(),
            },
            Err(_) => std::ptr::null_mut(),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_kv_close(store: *mut KV) {
    if !store.is_null() {
        unsafe { let _ = Box::from_raw(store); };
    }
}