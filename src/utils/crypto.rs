use cipher::{KeyIvInit, StreamCipher};
use md5::Md5;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::Rng;

type Aes256Ctr = ctr::Ctr128BE<aes::Aes256>;

pub fn generate_key() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut key = vec![0u8; 32];
    rng.fill(&mut key[..]);
    key
}

pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 32 {
        return Err("Key must be 32 bytes".to_string());
    }
    
    let mut rng = rand::thread_rng();
    let mut nonce = [0u8; 16];
    rng.fill(&mut nonce[..]);
    
    let mut cipher = Aes256Ctr::new(key.into(), (&nonce).into());
    let mut ciphertext = plaintext.to_vec();
    cipher.apply_keystream(&mut ciphertext);
    
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    if key.len() != 32 {
        return Err("Key must be 32 bytes".to_string());
    }
    
    if ciphertext.len() < 16 {
        return Err("Ciphertext too short".to_string());
    }
    
    let nonce = &ciphertext[..16];
    let data = &ciphertext[16..];
    
    let mut nonce_arr = [0u8; 16];
    nonce_arr.copy_from_slice(nonce);
    let mut cipher = Aes256Ctr::new(key.into(), (&nonce_arr).into());
    let mut plaintext = data.to_vec();
    cipher.apply_keystream(&mut plaintext);
    
    Ok(plaintext)
}

pub fn compute_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn compute_md5(data: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn encode_base64(data: &[u8]) -> String {
    BASE64.encode(data)
}

pub fn decode_base64(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    BASE64.decode(data)
}
