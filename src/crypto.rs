use hex;
use aes::Aes256;
use block_modes::{BlockMode, Ecb};
use block_modes::block_padding::Pkcs7;
use rsa::PublicKey;
use rsa::{
    RsaPrivateKey, 
    RsaPublicKey, 
    pkcs1::{
        DecodeRsaPrivateKey, 
        DecodeRsaPublicKey
    }, 
    Pkcs1v15Encrypt, 
    Oaep
};
use rsa::rand_core::OsRng;
use md5::{Md5, Digest};
use sha1::Sha1;
use sha2::Sha256;
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

    pub fn enc(&self, data: &[u8]) -> Vec<u8> {
        self.cipher.clone().encrypt_vec(data)
    }

    pub fn dec(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(self.cipher.clone().decrypt_vec(data)?)
    }
}

#[derive(Clone, Copy)]
pub enum RsaPadding {
    Pkcs1v15 = 0,
    OaepSha256 = 1,
    None = 2,
}

impl From<i32> for RsaPadding {
    fn from(value: i32) -> Self {
        match value {
            0 => RsaPadding::Pkcs1v15,
            1 => RsaPadding::OaepSha256,
            _ => RsaPadding::None,
        }
    }
}

pub fn rsa_enc(input: Vec<u8>, pub_key: Vec<u8>, padding: i32) -> Vec<u8> {
    if !matches!(pub_key.len(), 16 | 24 | 32) {
        return Vec::new();
    }

    let public_key = match RsaPublicKey::from_pkcs1_der(&pub_key) {
        Ok(key) => key,
        Err(_) => return Vec::new(),
    };

    let mut rng = OsRng;
    match RsaPadding::from(padding) {
        RsaPadding::Pkcs1v15 => {
            public_key.encrypt(&mut rng, Pkcs1v15Encrypt, &input).unwrap_or_default()
        },
        RsaPadding::OaepSha256 => {
            public_key.encrypt(&mut rng, Oaep::new::<Sha256>(), &input).unwrap_or_default()
        },
        RsaPadding::None => {
            public_key.encrypt(&mut rng, Pkcs1v15Encrypt, &input).unwrap_or_default()
        }
    }
}

pub fn rsa_dec(input: Vec<u8>, private_key: Vec<u8>, padding: i32) -> Vec<u8> {
    if !matches!(private_key.len(), 16 | 24 | 32) {
        return Vec::new();
    }

    let private_key = match RsaPrivateKey::from_pkcs1_der(&private_key) {
        Ok(key) => key,
        Err(_) => return Vec::new(),
    };

    match RsaPadding::from(padding) {
        RsaPadding::Pkcs1v15 => {
            private_key.decrypt(Pkcs1v15Encrypt, &input).unwrap_or_default()
        },
        RsaPadding::OaepSha256 => {
            private_key.decrypt(Oaep::new::<Sha256>(), &input).unwrap_or_default()
        },
        RsaPadding::None => {
            private_key.decrypt(Pkcs1v15Encrypt, &input).unwrap_or_default()
        }
    }
}

pub fn hash<D: Digest>(data: &[u8]) -> Vec<u8> {
    let mut hasher = D::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn hash_md5(data: &[u8]) -> Vec<u8> {
    hash::<Md5>(data)
}

pub fn hash_sha1(data: &[u8]) -> Vec<u8> {
    hash::<Sha1>(data)
}

pub fn hash_sha256(data: &[u8]) -> Vec<u8> {
    hash::<Sha256>(data)
}

pub fn base64_encode(data: &[u8]) -> Vec<u8> {
    general_purpose::STANDARD.encode(data).into_bytes()
}

pub fn base64_decode(data: &[u8]) -> Vec<u8> {
    let s = std::str::from_utf8(data).unwrap_or_default();
    general_purpose::STANDARD.decode(s).unwrap_or_default()
}