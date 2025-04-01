use hex;
use aes::Aes256;
use block_modes::{BlockMode, Ecb};
use block_modes::block_padding::Pkcs7;
use md5::{Md5, Digest};
use sha1::{Sha1, Digest};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use std::error::Error;

pub fn str2bytes(s: String) -> Vec<u8> {
    s.into_bytes()
}

pub fn bytes2str(v: Vec<u8>) -> Result<String, Box<dyn Error>> {
    String::from_utf8(v).map_err(|e| e.into())
}

pub fn hex2bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    hex::decode(hex).map_err(|e| e.into())
}

pub fn bytes2hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

type Aes256Ecb = Ecb<Aes256, Pkcs7>;

pub struct Aes256EcbPkcs5 {
    cipher: Aes256Ecb,
}

impl Aes256EcbPkcs5 {
    pub fn new(key: &[u8]) -> Result<Self, Box<dyn Error>> {
        if key.len() != 32 {
            return Err("Key must be 32 bytes (256 bits)".into());
        }
        Ok(Self {
            cipher: Aes256Ecb::new_from_slices(key, &[])?,
        })
    }

    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        self.cipher.encrypt_vec(data)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(self.cipher.decrypt_vec(data)?)
    }
}

pub fn hash_md5(data: &[u8]) -> Vec<u8> {
    let mut hasher = Md5::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn hash_sha1(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn hash_sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn base64_encode(data: &[u8]) -> Vec<u8> {
    general_purpose::STANDARD.encode(data).into_bytes()
}

pub fn base64_decode(data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let s = std::str::from_utf8(data)?;
    general_purpose::STANDARD.decode(s).map_err(|e| e.into())
}