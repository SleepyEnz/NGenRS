use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use tokio::runtime::Runtime;
use once_cell::sync::Lazy;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

#[unsafe(no_mangle)]
pub extern "C" fn http_get(url: *const c_char) -> *mut c_char {
    // Convert the C string to a Rust string
    let c_str = unsafe { CStr::from_ptr(url) };
    let url_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    // Perform the HTTP GET request using the global runtime
    let result = RUNTIME.block_on(async {
        match reqwest::get(url_str).await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text().await {
                        Ok(text) => CString::new(text).unwrap().into_raw(),
                        Err(_) => std::ptr::null_mut(),
                    }
                } else {
                    std::ptr::null_mut()
                }
            }
            Err(_) => std::ptr::null_mut(),
        }
    });

    result
}

#[unsafe(no_mangle)]
pub extern "C" fn free_c_string(s: *mut c_char) {
    if s.is_null() { return; }
    unsafe {
        // Convert the raw pointer back to a CString and immediately drop it
        drop(CString::from_raw(s));
    }
}