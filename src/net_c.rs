use crate::cc::{cstr_to_rust, rust_to_cstring};
use crate::net::http_get_async;
use std::os::raw::c_char;
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_http_get(url: *const c_char) -> *mut c_char {
    let url_str = match cstr_to_rust(url) {
        Some(s) => s,
        None => return std::ptr::null_mut(),
    };

    let result = RUNTIME.block_on(async {
        match http_get_async(url_str).await {
            Some(text) => rust_to_cstring(text),
            None => std::ptr::null_mut(),
        }
    });

    result
}