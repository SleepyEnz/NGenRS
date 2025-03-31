use tokio::runtime::Runtime;
use once_cell::sync::Lazy;
use crate::cc::{cstr_to_rust, rust_to_cstring};
use std::os::raw::c_char;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

pub async fn http_get_async(url: &str) -> Option<String> {
    match reqwest::get(url).await {
        Ok(response) => {
            if response.status().is_success() {
                response.text().await.ok()
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn http_get_unsafe(url: *const c_char) -> *mut c_char {
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