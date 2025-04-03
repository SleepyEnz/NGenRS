use std::os::raw::{c_char, c_void};
use std::collections::HashMap;
use crate::c::util::{cstr_to_rust, rust_to_cstr, ngenrs_free_ptr, box_into_raw_new};
use crate::core::net::{HttpClient, HttpResponse};
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;
use serde_json::Value;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

// Client management
#[unsafe(no_mangle)]
pub extern "C"
fn ngenrs_http_client_new(ca_cert_path: *const c_char) -> *mut c_void {
    let ca_path = if !ca_cert_path.is_null() {
        let path_str = cstr_to_rust(ca_cert_path).unwrap();
        Some(std::path::Path::new(path_str))
    } else {
        None
    };

    box_into_raw_new(
        HttpClient::new(ca_path).expect("Failed to create HTTP client")
    ) as *mut c_void
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_http_client_free(client: *mut c_void) {
    ngenrs_free_ptr(client)
}

// Common response handling
unsafe fn handle_response(resp: Result<HttpResponse, Box<dyn std::error::Error>>) -> *mut c_void {
    match resp {
        Ok(resp) => box_into_raw_new(resp) as *mut c_void,
        Err(_) => std::ptr::null_mut(),
    }
}

// HTTP GET
#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_http_get(
    client: *const c_void,
    url: *const c_char,
    headers: *const HashMap<*const c_char, *const c_char>,
    headers_len: usize,
    body: *const c_char,
) -> *mut c_void {
    let client = unsafe { &*(client as *const HttpClient) };
    let url = cstr_to_rust(url).unwrap_or_default();
    let headers = unsafe { convert_c_headers(headers, headers_len) };
    let body = if !body.is_null() {
        Some(cstr_to_rust(body).unwrap_or_default())
    } else {
        None
    };

    let result = RUNTIME.block_on(async {
        client.get(&url, headers, body).await
    });

    unsafe { handle_response(result) }
}

// HTTP POST
#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_http_post(
    client: *const c_void,
    url: *const c_char,
    headers: *const HashMap<*const c_char, *const c_char>,
    headers_len: usize,
    body: *const c_char,
    json: *const c_char,
) -> *mut c_void {
    let client = unsafe { &*(client as *const HttpClient) };
    let url = cstr_to_rust(url).unwrap_or_default();
    let headers = unsafe { convert_c_headers(headers, headers_len) };
    let body = if !body.is_null() {
        Some(cstr_to_rust(body).unwrap_or_default())
    } else {
        None
    };
    let json_map = if !json.is_null() {
        Some(unsafe { parse_json_map(json) })
    } else {
        None
    };

    let result = RUNTIME.block_on(async {
        client.post(&url, headers, body, json_map).await
    });

    unsafe { handle_response(result) }
}

// Response handling
#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_http_response_free(resp: *mut c_void) {
    ngenrs_free_ptr(resp)
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_http_response_status(resp: *const c_void) -> u16 {
    let resp = unsafe { &*(resp as *const HttpResponse) };
    resp.status.as_u16()
}

#[unsafe(no_mangle)]
pub extern "C"
fn ngenrs_http_response_body(resp: *const c_void, out_len: *mut usize) -> *mut c_char {
    let resp = unsafe { &*(resp as *const HttpResponse) };
    let body_str = resp.body.clone().unwrap_or_default();
    let ptr = rust_to_cstr(body_str.clone());
    if !out_len.is_null() {
        unsafe { *out_len = body_str.len() };
    }
    ptr
}

// Helper functions
unsafe fn convert_c_headers(
    headers: *const HashMap<*const c_char, *const c_char>,
    len: usize,
) -> Option<HashMap<String, String>> {
    if headers.is_null() {
        return None;
    }
    let mut map = HashMap::new();
    let headers_slice = unsafe { std::slice::from_raw_parts(headers, len) };
    
    for header_map in headers_slice {
        for (k, v) in header_map.iter() {
            if let (Some(key), Some(value)) = (cstr_to_rust(*k), cstr_to_rust(*v)) {
                map.insert(key.to_string(), value.to_string());
            }
        }
    }
    Some(map)
}

unsafe fn parse_json_map(json: *const c_char) -> HashMap<String, Value> {
    let json_str = cstr_to_rust(json).unwrap_or_default();
    serde_json::from_str(&json_str).unwrap_or_default()
}