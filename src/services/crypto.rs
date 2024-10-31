use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key
};

pub struct CryptoService {}

impl CryptoService {
    pub fn generate_key() -> Key<Aes256Gcm> {
        Aes256Gcm::generate_key(&mut OsRng)
    }

    fn generate_nonce() -> Nonce<Aes256Gcm> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        nonce
    }

    pub fn encrypt(&self, data: &str, key: &Key<Aes256Gcm>) -> Result<(String, Nonce<Aes256Gcm>), String> {
        let cipher = Aes256Gcm::new(key);
        let nonce = CryptoService::generate_nonce();
        let ciphertext = cipher.encrypt(&nonce, data.as_ref()).expect("encryption failure!");
        Ok((hex::encode(ciphertext), nonce))
    }

    pub fn decrypt(ciphertext: &str, key: &Key<Aes256Gcm>, nonce: &Nonce<Aes256Gcm>) -> Result<String, String> {
        let cipher = Aes256Gcm::new(key);
        let ciphertext = hex::decode(ciphertext).expect("decoding failure!");
        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).expect("decryption failure!");
        String::from_utf8(plaintext).map_err(|e| e.to_string())
    }

    pub fn hex_to_key(text_key: &str) -> Key<Aes256Gcm> {
        let bytes = hex::decode(text_key).expect("decoding failure!");
        Key::<Aes256Gcm>::clone_from_slice(&bytes)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let key = CryptoService::generate_key();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_generate_nonce() {
        let nonce = CryptoService::generate_nonce();
        assert_eq!(nonce.len(), 12);
    }

    #[test]
    fn test_encryption_decryption() {
        let mut crypto_service = CryptoService{};
        let key = CryptoService::generate_key();
        let data = "secret data";
        let (encrypted, nonce) = match crypto_service.encrypt(data, &key) {
            Ok((ciphertext, nonce)) => (ciphertext, nonce),
            Err(e) => {
                println!("Encryption failed: {}", e);
                return;
            }
        };
        let decrypted = CryptoService::decrypt(&encrypted, &key, &nonce).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_hex_to_key() {
        let key = CryptoService::generate_key();
        let hex_key = hex::encode(key.as_slice());
        let key_from_hex = CryptoService::hex_to_key(&hex_key);
        assert_eq!(key.as_slice(), key_from_hex.as_slice());
    }
}
