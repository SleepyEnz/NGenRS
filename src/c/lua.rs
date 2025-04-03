use std::ffi::{c_char, c_void};
use crate::c::util::{cstr_to_rust, rust_to_cstr, ngenrs_free_ptr, box_into_raw_new};
use crate::core::lua::LuaBridge;

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_lua_bridge_init() -> *mut c_void {
    match LuaBridge::new() {
        Ok(bridge) => box_into_raw_new(bridge) as *mut c_void,
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_lua_bridge_release(bridge: *mut c_void) {
    ngenrs_free_ptr(bridge)
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_lua_load_file(
    bridge: *mut c_void,
    path: *const c_char,
) -> bool {
    if bridge.is_null() || path.is_null() {
        return false;
    }
    let bridge = unsafe { &*(bridge as *mut LuaBridge) };
    let path_str = match cstr_to_rust(path) {
        Some(s) => s,
        None => return false,
    };
    bridge.load_file(&path_str).is_ok()
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_lua_load_string(
    bridge: *mut c_void,
    script: *const c_char,
) -> bool {
    if bridge.is_null() || script.is_null() {
        return false;
    }
    let bridge = unsafe { &*(bridge as *mut LuaBridge) };
    let script_str = match cstr_to_rust(script) {
        Some(s) => s,
        None => return false,
    };
    bridge.load_string(&script_str).is_ok()
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_lua_call_function(
    bridge: *mut c_void,
    func_name: *const c_char,
    args: *const c_char,
) -> *mut c_char {
    if bridge.is_null() || func_name.is_null() {
        return std::ptr::null_mut();
    }
    let bridge = unsafe { &*(bridge as *mut LuaBridge) };
    let func_name_str = match cstr_to_rust(func_name) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };
    let args_str = if !args.is_null() {
        match cstr_to_rust(args) {
            Some(s) => s,
            None => return std::ptr::null_mut(),
        }
    } else {
        ""
    };

    match bridge.call_function::<_, String>(func_name_str, args_str) {
        Ok(result) => rust_to_cstr(result),
        Err(_) => std::ptr::null_mut(),
    }
}