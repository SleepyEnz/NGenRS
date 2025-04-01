use std::ffi::CString;
use std::os::raw::c_int;
use crate::zip::{CompressionFormat, compress, decompress};
use crate::cc::{cbytes_to_rust, rust_to_cbytes};

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_z_compress(
    input: *const u8,
    input_len: usize,
    output: *mut *mut u8,
    output_len: *mut usize,
    format: c_int,
) -> *mut u8 {
    let format = match format {
        0 => CompressionFormat::Gzip,
        1 => CompressionFormat::Zlib,
        2 => CompressionFormat::Raw,
        _ => return std::ptr::null_mut(),
    };

    if input.is_null() || output.is_null() || output_len.is_null() {
        return std::ptr::null_mut();
    }

    let input_slice = match cbytes_to_rust(input, input_len) {
        Some(slice) => slice,
        None => return CString::new("Invalid input buffer").unwrap().into_raw() as *mut u8,
    };
    let reader = std::io::Cursor::new(input_slice);

    match compress(reader, format) {
        Ok(result) => {
            let (ptr, len) = rust_to_cbytes(result);
            unsafe {
                *output = ptr;
                *output_len = len;
            }
            std::ptr::null_mut()
        }
        Err(e) => CString::new(e.to_string()).unwrap().into_raw() as *mut u8,
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_z_decompress(
    input: *const u8,
    input_len: usize,
    output: *mut *mut u8,
    output_len: *mut usize,
    format: c_int,
) -> *mut u8 {
    let format = match format {
        0 => CompressionFormat::Gzip,
        1 => CompressionFormat::Zlib,
        2 => CompressionFormat::Raw,
        _ => return std::ptr::null_mut(),
    };

    if input.is_null() || output.is_null() || output_len.is_null() {
        return std::ptr::null_mut();
    }

    let input_slice = match cbytes_to_rust(input, input_len) {
        Some(slice) => slice,
        None => return CString::new("Invalid input buffer").unwrap().into_raw() as *mut u8,
    };
    let reader = std::io::Cursor::new(input_slice);

    match decompress(reader, format) {
        Ok(result) => {
            let (ptr, len) = rust_to_cbytes(result);
            unsafe {
                *output = ptr;
                *output_len = len;
            }
            std::ptr::null_mut()
        }
        Err(e) => CString::new(e.to_string()).unwrap().into_raw() as *mut u8,
    }
}