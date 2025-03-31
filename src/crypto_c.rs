use std::os::raw::c_char;
use crate::cc::{cbytes_to_rust, rust_to_cbytes, ngenrs_free_ptr};
use crate::crypto::Aes256EcbPkcs5;

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_crypto_aes256_ecb_pkcs5_init(key: *const c_char, key_len: usize) -> *mut Aes256EcbPkcs5 {
    if key.is_null() {
        return std::ptr::null_mut();
    }

    let key_bytes = unsafe { slice::from_raw_parts(key as *const u8, key_len) };
    match Aes256EcbPkcs5::new(key_bytes) {
        Ok(cipher) => Box::into_raw(Box::new(cipher)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_crypto_aes256_ecb_pkcs5_encrypt(
    cipher: *mut Aes256EcbPkcs5,
    data: *const u8,
    data_len: usize,
    out_len: *mut usize,
) -> *mut u8 {
    if cipher.is_null() || data.is_null() {
        return std::ptr::null_mut();
    }

    let data_bytes = unsafe { cbytes_to_rust(data, data_len) };
    let cipher_ref = unsafe { &*cipher };
    let encrypted = cipher_ref.encrypt(data_bytes);
    let (ptr, len) = rust_to_cbytes(encrypted);

    unsafe { *out_len = len };
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_crypto_aes256_ecb_pkcs5_decrypt(
    cipher: *mut Aes256EcbPkcs5,
    data: *const u8,
    data_len: usize,
    out_len: *mut usize,
) -> *mut u8 {
    if cipher.is_null() || data.is_null() {
        return std::ptr::null_mut();
    }

    let data_bytes = unsafe { cbytes_to_rust(data, data_len) };
    let cipher_ref = unsafe { &*cipher };
    match cipher_ref.decrypt(data_bytes) {
        Ok(decrypted) => {
            let (ptr, len) = rust_to_cbytes(decrypted);
            unsafe { *out_len = len };
            ptr
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" 
fn ngenrs_crypto_aes256_ecb_pkcs5_release(cipher: *mut Aes256EcbPkcs5) {
    ngenrs_free_ptr(cipher);
}