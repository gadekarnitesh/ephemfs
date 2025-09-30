// Example: Custom Cipher Implementation for SecretFS
// 
// This example shows how to implement your own encryption cipher
// for SecretFS by implementing the SecretCipher trait.

use std::env;
use secretfs::encryption::{SecretCipher, EncryptionError};

/// Example: AES-like cipher (simplified for demonstration)
/// 
/// ⚠️ WARNING: This is a simplified example for demonstration only!
/// In production, use proper cryptographic libraries like:
/// - `aes-gcm` for AES-256-GCM encryption
/// - `chacha20poly1305` for ChaCha20-Poly1305
/// - `ring` for various cryptographic primitives
pub struct CustomAESCipher {
    key: [u8; 32], // 256-bit key
}

impl CustomAESCipher {
    pub fn new(key: &[u8]) -> Result<Self, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKey(
                "AES-256 requires a 32-byte key".to_string()
            ));
        }
        
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(key);
        
        Ok(Self { key: key_array })
    }
    
    pub fn from_env() -> Result<Self, EncryptionError> {
        let key_hex = env::var("SECRETFS_AES_KEY")
            .map_err(|_| EncryptionError::InvalidKey(
                "SECRETFS_AES_KEY environment variable not set".to_string()
            ))?;
        
        let key_bytes = hex::decode(&key_hex)
            .map_err(|_| EncryptionError::InvalidKey(
                "SECRETFS_AES_KEY must be valid hex".to_string()
            ))?;
        
        Self::new(&key_bytes)
    }
}

impl SecretCipher for CustomAESCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // ⚠️ SIMPLIFIED EXAMPLE - NOT SECURE!
        // In production, use proper AES-GCM implementation:
        //
        // use aes_gcm::{Aes256Gcm, Key, Nonce};
        // use aes_gcm::aead::{Aead, NewAead};
        //
        // let cipher = Aes256Gcm::new(Key::from_slice(&self.key));
        // let nonce = Nonce::from_slice(b"unique nonce"); // Use random nonce!
        // cipher.encrypt(nonce, plaintext)
        //     .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))
        
        // For demo: simple XOR with rotating key
        let mut encrypted = Vec::with_capacity(plaintext.len());
        for (i, &byte) in plaintext.iter().enumerate() {
            let key_byte = self.key[i % self.key.len()];
            encrypted.push(byte ^ key_byte);
        }
        Ok(encrypted)
    }
    
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // ⚠️ SIMPLIFIED EXAMPLE - NOT SECURE!
        // In production, use proper AES-GCM implementation
        
        // For demo: XOR is symmetric
        let mut decrypted = Vec::with_capacity(ciphertext.len());
        for (i, &byte) in ciphertext.iter().enumerate() {
            let key_byte = self.key[i % self.key.len()];
            decrypted.push(byte ^ key_byte);
        }
        Ok(decrypted)
    }
    
    fn cipher_info(&self) -> String {
        "CustomAESCipher (Demo XOR with 256-bit key) - ⚠️ EXAMPLE ONLY!".to_string()
    }
}

/// Example: Base64 "encryption" (encoding only - NOT secure!)
pub struct Base64Cipher;

impl Base64Cipher {
    pub fn new() -> Self {
        Self
    }
}

impl SecretCipher for Base64Cipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // ⚠️ NOT ENCRYPTION - just encoding!
        use base64::{Engine as _, engine::general_purpose};
        let encoded = general_purpose::STANDARD.encode(plaintext);
        Ok(encoded.into_bytes())
    }
    
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        use base64::{Engine as _, engine::general_purpose};
        let encoded_str = String::from_utf8(ciphertext.to_vec())
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;
        
        general_purpose::STANDARD.decode(&encoded_str)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))
    }
    
    fn cipher_info(&self) -> String {
        "Base64Cipher (encoding only) - ⚠️ NOT SECURE - FOR TESTING ONLY!".to_string()
    }
}

/// Example: ROT13 cipher (for demonstration)
pub struct ROT13Cipher;

impl ROT13Cipher {
    pub fn new() -> Self {
        Self
    }
    
    fn rot13_byte(byte: u8) -> u8 {
        match byte {
            b'A'..=b'Z' => ((byte - b'A' + 13) % 26) + b'A',
            b'a'..=b'z' => ((byte - b'a' + 13) % 26) + b'a',
            _ => byte,
        }
    }
}

impl SecretCipher for ROT13Cipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        Ok(plaintext.iter().map(|&b| Self::rot13_byte(b)).collect())
    }
    
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // ROT13 is symmetric
        Ok(ciphertext.iter().map(|&b| Self::rot13_byte(b)).collect())
    }
    
    fn cipher_info(&self) -> String {
        "ROT13Cipher - ⚠️ TRIVIAL ENCRYPTION - FOR DEMO ONLY!".to_string()
    }
}

// Example usage in main.rs:
//
// mod custom_cipher;
// use custom_cipher::{CustomAESCipher, Base64Cipher, ROT13Cipher};
//
// fn create_custom_cipher() -> Box<dyn SecretCipher> {
//     let cipher_type = std::env::var("SECRETFS_CIPHER_TYPE")
//         .unwrap_or_else(|_| "default".to_string())
//         .to_lowercase();
//     
//     match cipher_type.as_str() {
//         "aes" => {
//             match CustomAESCipher::from_env() {
//                 Ok(cipher) => Box::new(cipher),
//                 Err(e) => {
//                     eprintln!("Failed to create AES cipher: {}", e);
//                     std::process::exit(1);
//                 }
//             }
//         },
//         "base64" => Box::new(Base64Cipher::new()),
//         "rot13" => Box::new(ROT13Cipher::new()),
//         "plaintext" => Box::new(PlaintextCipher::new()),
//         "default" | _ => Box::new(DefaultCipher::from_env()),
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_custom_aes_cipher() {
        let key = [0u8; 32]; // All zeros for test
        let cipher = CustomAESCipher::new(&key).unwrap();
        
        let plaintext = b"hello world";
        let encrypted = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_base64_cipher() {
        let cipher = Base64Cipher::new();
        
        let plaintext = b"hello world";
        let encrypted = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        
        assert_eq!(decrypted, plaintext);
        // Base64 encoding should be different from original
        assert_ne!(encrypted, plaintext);
    }
    
    #[test]
    fn test_rot13_cipher() {
        let cipher = ROT13Cipher::new();
        
        let plaintext = b"Hello World";
        let encrypted = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        
        assert_eq!(decrypted, plaintext);
        assert_eq!(encrypted, b"Uryyb Jbeyq");
    }
}
