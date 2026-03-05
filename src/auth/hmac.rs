use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

type HmacSha256 = Hmac<Sha256>;

pub fn generate_hmac_signature(message: &str, key: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    BASE64.encode(result.into_bytes())
}

pub fn verify_hmac_signature(message: &str, key: &[u8], signature: &str) -> bool {
    let expected = generate_hmac_signature(message, key);
    expected == signature
}

pub fn generate_nonce() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.gen();
    BASE64.encode(bytes)
}

pub fn compute_hash(data: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}
