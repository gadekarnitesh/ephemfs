use std::env;
use std::fs;
use rsa::{RsaPrivateKey, RsaPublicKey, Pkcs1v15Encrypt};
use rsa::pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey};
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, DecodePrivateKey, DecodePublicKey};
use rsa::traits::{PublicKeyParts, PrivateKeyParts};
use rand::rngs::OsRng;
use base64::{Engine as _, engine::general_purpose};

/// Errors that can occur during asymmetric encryption operations
#[derive(Debug)]
pub enum AsymmetricError {
    KeyGenerationError(String),
    KeyLoadError(String),
    EncryptionError(String),
    DecryptionError(String),
    InvalidKeyFormat(String),
    FileError(String),
    ConfigurationError(String),
}

impl std::fmt::Display for AsymmetricError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AsymmetricError::KeyGenerationError(msg) => write!(f, "Key generation error: {}", msg),
            AsymmetricError::KeyLoadError(msg) => write!(f, "Key load error: {}", msg),
            AsymmetricError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            AsymmetricError::DecryptionError(msg) => write!(f, "Decryption error: {}", msg),
            AsymmetricError::InvalidKeyFormat(msg) => write!(f, "Invalid key format: {}", msg),
            AsymmetricError::FileError(msg) => write!(f, "File error: {}", msg),
            AsymmetricError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for AsymmetricError {}

/// RSA key pair for asymmetric encryption
#[derive(Clone)]
pub struct RsaKeyPair {
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
}

impl RsaKeyPair {
    /// Generate a new RSA key pair with specified bit size
    pub fn generate(bits: usize) -> Result<Self, AsymmetricError> {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, bits)
            .map_err(|e| AsymmetricError::KeyGenerationError(format!("Failed to generate private key: {}", e)))?;
        
        let public_key = RsaPublicKey::from(&private_key);
        
        Ok(RsaKeyPair {
            private_key,
            public_key,
        })
    }
    
    /// Load key pair from PEM files
    pub fn from_pem_files(private_key_path: &str, public_key_path: &str) -> Result<Self, AsymmetricError> {
        let private_pem = fs::read_to_string(private_key_path)
            .map_err(|e| AsymmetricError::FileError(format!("Failed to read private key file {}: {}", private_key_path, e)))?;
        
        let public_pem = fs::read_to_string(public_key_path)
            .map_err(|e| AsymmetricError::FileError(format!("Failed to read public key file {}: {}", public_key_path, e)))?;
        
        Self::from_pem_strings(&private_pem, &public_pem)
    }
    
    /// Load key pair from PEM strings
    pub fn from_pem_strings(private_pem: &str, public_pem: &str) -> Result<Self, AsymmetricError> {
        // Try PKCS#8 format first, then PKCS#1
        let private_key = DecodePrivateKey::from_pkcs8_pem(private_pem)
            .or_else(|_| DecodeRsaPrivateKey::from_pkcs1_pem(private_pem))
            .map_err(|e| AsymmetricError::KeyLoadError(format!("Failed to decode private key: {}", e)))?;
        
        let public_key = DecodePublicKey::from_public_key_pem(public_pem)
            .or_else(|_| DecodeRsaPublicKey::from_pkcs1_pem(public_pem))
            .map_err(|e| AsymmetricError::KeyLoadError(format!("Failed to decode public key: {}", e)))?;
        
        Ok(RsaKeyPair {
            private_key,
            public_key,
        })
    }
    
    /// Save key pair to PEM files
    pub fn save_to_pem_files(&self, private_key_path: &str, public_key_path: &str) -> Result<(), AsymmetricError> {
        let private_pem = self.private_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| AsymmetricError::KeyGenerationError(format!("Failed to encode private key: {}", e)))?;
        
        let public_pem = self.public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| AsymmetricError::KeyGenerationError(format!("Failed to encode public key: {}", e)))?;
        
        fs::write(private_key_path, private_pem)
            .map_err(|e| AsymmetricError::FileError(format!("Failed to write private key file {}: {}", private_key_path, e)))?;
        
        fs::write(public_key_path, public_pem)
            .map_err(|e| AsymmetricError::FileError(format!("Failed to write public key file {}: {}", public_key_path, e)))?;
        
        Ok(())
    }
    
    /// Get public key as PEM string
    pub fn public_key_pem(&self) -> Result<String, AsymmetricError> {
        self.public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| AsymmetricError::KeyGenerationError(format!("Failed to encode public key: {}", e)))
    }
    
    /// Get private key as PEM string
    pub fn private_key_pem(&self) -> Result<String, AsymmetricError> {
        let pem = self.private_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| AsymmetricError::KeyGenerationError(format!("Failed to encode private key: {}", e)))?;
        Ok(pem.to_string())
    }
}

/// Asymmetric encryption manager
pub struct AsymmetricEncryption {
    public_key: RsaPublicKey,
    key_info: String,
}

impl AsymmetricEncryption {
    /// Create new asymmetric encryption with public key only (for encryption)
    pub fn new_with_public_key(public_key: RsaPublicKey) -> Self {
        let key_size = public_key.size() * 8; // Convert bytes to bits
        let key_info = format!("RSA-{} (Public Key Only - Encryption Only)", key_size);
        
        Self {
            public_key,
            key_info,
        }
    }
    
    /// Load from environment configuration
    pub fn from_env() -> Result<Self, AsymmetricError> {
        // Check for public key in environment
        if let Ok(public_key_pem) = env::var("SECRETFS_PUBLIC_KEY_PEM") {
            let public_key = DecodePublicKey::from_public_key_pem(&public_key_pem)
                .or_else(|_| DecodeRsaPublicKey::from_pkcs1_pem(&public_key_pem))
                .map_err(|e| AsymmetricError::KeyLoadError(format!("Failed to decode public key from environment: {}", e)))?;
            
            return Ok(Self::new_with_public_key(public_key));
        }
        
        // Check for public key file path
        if let Ok(public_key_path) = env::var("SECRETFS_PUBLIC_KEY_FILE") {
            let public_key_pem = fs::read_to_string(&public_key_path)
                .map_err(|e| AsymmetricError::FileError(format!("Failed to read public key file {}: {}", public_key_path, e)))?;
            
            let public_key = DecodePublicKey::from_public_key_pem(&public_key_pem)
                .or_else(|_| DecodeRsaPublicKey::from_pkcs1_pem(&public_key_pem))
                .map_err(|e| AsymmetricError::KeyLoadError(format!("Failed to decode public key from file: {}", e)))?;
            
            return Ok(Self::new_with_public_key(public_key));
        }
        
        Err(AsymmetricError::ConfigurationError(
            "No public key configuration found. Set SECRETFS_PUBLIC_KEY_PEM or SECRETFS_PUBLIC_KEY_FILE".to_string()
        ))
    }
    
    /// Encrypt data with public key
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, AsymmetricError> {
        let mut rng = OsRng;
        
        // RSA can only encrypt data smaller than the key size minus padding
        // For RSA-2048, max plaintext is ~245 bytes with PKCS1v15 padding
        let max_chunk_size = self.public_key.size() - 11; // PKCS1v15 padding overhead
        
        if plaintext.len() <= max_chunk_size {
            // Single chunk encryption
            let ciphertext = self.public_key.encrypt(&mut rng, Pkcs1v15Encrypt, plaintext)
                .map_err(|e| AsymmetricError::EncryptionError(format!("RSA encryption failed: {}", e)))?;
            
            Ok(ciphertext)
        } else {
            // Multi-chunk encryption for larger data
            let mut encrypted_chunks = Vec::new();
            
            for chunk in plaintext.chunks(max_chunk_size) {
                let encrypted_chunk = self.public_key.encrypt(&mut rng, Pkcs1v15Encrypt, chunk)
                    .map_err(|e| AsymmetricError::EncryptionError(format!("RSA chunk encryption failed: {}", e)))?;
                
                // Store chunk size (2 bytes) + encrypted chunk
                encrypted_chunks.extend_from_slice(&(encrypted_chunk.len() as u16).to_be_bytes());
                encrypted_chunks.extend_from_slice(&encrypted_chunk);
            }
            
            Ok(encrypted_chunks)
        }
    }
    
    /// Encrypt and encode as base64
    pub fn encrypt_base64(&self, plaintext: &[u8]) -> Result<String, AsymmetricError> {
        let ciphertext = self.encrypt(plaintext)?;
        Ok(general_purpose::STANDARD.encode(ciphertext))
    }
    
    /// Get encryption info
    pub fn encryption_info(&self) -> &str {
        &self.key_info
    }
}

/// Asymmetric decryption manager (for applications with private key)
pub struct AsymmetricDecryption {
    private_key: RsaPrivateKey,
    key_info: String,
}

impl AsymmetricDecryption {
    /// Create new asymmetric decryption with private key
    pub fn new_with_private_key(private_key: RsaPrivateKey) -> Self {
        let key_size = private_key.size() * 8; // Convert bytes to bits
        let key_info = format!("RSA-{} (Private Key - Decryption Capable)", key_size);
        
        Self {
            private_key,
            key_info,
        }
    }
    
    /// Load from environment configuration (for applications)
    pub fn from_env() -> Result<Self, AsymmetricError> {
        // Check for private key in environment
        if let Ok(private_key_pem) = env::var("SECRETFS_PRIVATE_KEY_PEM") {
            let private_key = DecodePrivateKey::from_pkcs8_pem(&private_key_pem)
                .or_else(|_| DecodeRsaPrivateKey::from_pkcs1_pem(&private_key_pem))
                .map_err(|e| AsymmetricError::KeyLoadError(format!("Failed to decode private key from environment: {}", e)))?;
            
            return Ok(Self::new_with_private_key(private_key));
        }
        
        // Check for private key file path
        if let Ok(private_key_path) = env::var("SECRETFS_PRIVATE_KEY_FILE") {
            let private_key_pem = fs::read_to_string(&private_key_path)
                .map_err(|e| AsymmetricError::FileError(format!("Failed to read private key file {}: {}", private_key_path, e)))?;
            
            let private_key = DecodePrivateKey::from_pkcs8_pem(&private_key_pem)
                .or_else(|_| DecodeRsaPrivateKey::from_pkcs1_pem(&private_key_pem))
                .map_err(|e| AsymmetricError::KeyLoadError(format!("Failed to decode private key from file: {}", e)))?;
            
            return Ok(Self::new_with_private_key(private_key));
        }
        
        Err(AsymmetricError::ConfigurationError(
            "No private key configuration found. Set SECRETFS_PRIVATE_KEY_PEM or SECRETFS_PRIVATE_KEY_FILE".to_string()
        ))
    }
    
    /// Decrypt data with private key
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, AsymmetricError> {
        let key_size = self.private_key.size();
        
        if ciphertext.len() == key_size {
            // Single chunk decryption
            let plaintext = self.private_key.decrypt(Pkcs1v15Encrypt, ciphertext)
                .map_err(|e| AsymmetricError::DecryptionError(format!("RSA decryption failed: {}", e)))?;
            
            Ok(plaintext)
        } else {
            // Multi-chunk decryption
            let mut decrypted_data = Vec::new();
            let mut offset = 0;
            
            while offset < ciphertext.len() {
                if offset + 2 > ciphertext.len() {
                    return Err(AsymmetricError::DecryptionError("Invalid encrypted data format".to_string()));
                }
                
                // Read chunk size
                let chunk_size = u16::from_be_bytes([ciphertext[offset], ciphertext[offset + 1]]) as usize;
                offset += 2;
                
                if offset + chunk_size > ciphertext.len() {
                    return Err(AsymmetricError::DecryptionError("Invalid chunk size in encrypted data".to_string()));
                }
                
                // Decrypt chunk
                let encrypted_chunk = &ciphertext[offset..offset + chunk_size];
                let decrypted_chunk = self.private_key.decrypt(Pkcs1v15Encrypt, encrypted_chunk)
                    .map_err(|e| AsymmetricError::DecryptionError(format!("RSA chunk decryption failed: {}", e)))?;
                
                decrypted_data.extend_from_slice(&decrypted_chunk);
                offset += chunk_size;
            }
            
            Ok(decrypted_data)
        }
    }
    
    /// Decrypt from base64
    pub fn decrypt_base64(&self, ciphertext_base64: &str) -> Result<Vec<u8>, AsymmetricError> {
        let ciphertext = general_purpose::STANDARD.decode(ciphertext_base64)
            .map_err(|e| AsymmetricError::DecryptionError(format!("Base64 decode failed: {}", e)))?;
        
        self.decrypt(&ciphertext)
    }
    
    /// Get decryption info
    pub fn decryption_info(&self) -> &str {
        &self.key_info
    }
}

/// Utility functions for key management
pub mod key_utils {
    use super::*;
    
    /// Generate and save a new RSA key pair
    pub fn generate_key_pair(bits: usize, private_key_path: &str, public_key_path: &str) -> Result<(), AsymmetricError> {
        println!("🔑 Generating RSA-{} key pair...", bits);
        
        let key_pair = RsaKeyPair::generate(bits)?;
        key_pair.save_to_pem_files(private_key_path, public_key_path)?;
        
        println!("✅ Key pair generated successfully:");
        println!("   Private key: {}", private_key_path);
        println!("   Public key: {}", public_key_path);
        println!("⚠️  Keep the private key secure and distribute only the public key!");
        
        Ok(())
    }
    
    /// Display key information
    pub fn display_key_info(key_path: &str) -> Result<(), AsymmetricError> {
        let key_pem = fs::read_to_string(key_path)
            .map_err(|e| AsymmetricError::FileError(format!("Failed to read key file {}: {}", key_path, e)))?;
        
        if key_pem.contains("PRIVATE KEY") {
            let private_key: RsaPrivateKey = DecodePrivateKey::from_pkcs8_pem(&key_pem)
                .or_else(|_| DecodeRsaPrivateKey::from_pkcs1_pem(&key_pem))
                .map_err(|e| AsymmetricError::KeyLoadError(format!("Failed to decode private key: {}", e)))?;
            
            println!("🔐 Private Key Information:");
            println!("   File: {}", key_path);
            println!("   Size: {} bits", private_key.size() * 8);
            println!("   Type: RSA Private Key");
            println!("   ⚠️  This key can decrypt secrets - keep it secure!");
        } else if key_pem.contains("PUBLIC KEY") {
            let public_key: RsaPublicKey = DecodePublicKey::from_public_key_pem(&key_pem)
                .or_else(|_| DecodeRsaPublicKey::from_pkcs1_pem(&key_pem))
                .map_err(|e| AsymmetricError::KeyLoadError(format!("Failed to decode public key: {}", e)))?;
            
            println!("🔓 Public Key Information:");
            println!("   File: {}", key_path);
            println!("   Size: {} bits", public_key.size() * 8);
            println!("   Type: RSA Public Key");
            println!("   ✅ This key is safe to distribute for encryption");
        } else {
            return Err(AsymmetricError::InvalidKeyFormat("Unknown key format".to_string()));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_generation() {
        let key_pair = RsaKeyPair::generate(2048).unwrap();
        assert_eq!(key_pair.private_key.size(), 256); // 2048 bits = 256 bytes
        assert_eq!(key_pair.public_key.size(), 256);
    }
    
    #[test]
    fn test_encryption_decryption() {
        let key_pair = RsaKeyPair::generate(2048).unwrap();
        let encryption = AsymmetricEncryption::new_with_public_key(key_pair.public_key.clone());
        let decryption = AsymmetricDecryption::new_with_private_key(key_pair.private_key);
        
        let plaintext = b"Hello, SecretFS with RSA encryption!";
        let ciphertext = encryption.encrypt(plaintext).unwrap();
        let decrypted = decryption.decrypt(&ciphertext).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }
    
    #[test]
    fn test_large_data_encryption() {
        let key_pair = RsaKeyPair::generate(2048).unwrap();
        let encryption = AsymmetricEncryption::new_with_public_key(key_pair.public_key.clone());
        let decryption = AsymmetricDecryption::new_with_private_key(key_pair.private_key);
        
        // Test with data larger than RSA key size
        let plaintext = vec![42u8; 1000]; // 1KB of data
        let ciphertext = encryption.encrypt(&plaintext).unwrap();
        let decrypted = decryption.decrypt(&ciphertext).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
    
    #[test]
    fn test_base64_encoding() {
        let key_pair = RsaKeyPair::generate(2048).unwrap();
        let encryption = AsymmetricEncryption::new_with_public_key(key_pair.public_key.clone());
        let decryption = AsymmetricDecryption::new_with_private_key(key_pair.private_key);
        
        let plaintext = b"Base64 encoded secret";
        let ciphertext_base64 = encryption.encrypt_base64(plaintext).unwrap();
        let decrypted = decryption.decrypt_base64(&ciphertext_base64).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }
}
