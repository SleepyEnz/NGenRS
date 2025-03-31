use aes::Aes256;
use block_modes::{BlockMode, Ecb};
use block_modes::block_padding::Pkcs7;
use std::error::Error;

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