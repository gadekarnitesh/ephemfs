use std::error::Error;
use std::fmt;
use std::env;
use crate::asymmetric_encryption::{AsymmetricEncryption, AsymmetricError};

/// Custom error type for encryption operations
#[derive(Debug)]
pub enum EncryptionError {
    EncryptionFailed(String),
    DecryptionFailed(String),
    InvalidKey(String),
    InvalidData(String),
}

impl fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EncryptionError::EncryptionFailed(msg) => write!(f, "Encryption failed: {}", msg),
            EncryptionError::DecryptionFailed(msg) => write!(f, "Decryption failed: {}", msg),
            EncryptionError::InvalidKey(msg) => write!(f, "Invalid key: {}", msg),
            EncryptionError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl Error for EncryptionError {}

/// Trait for secret encryption and decryption
/// 
/// This trait allows users to implement custom encryption methods
/// or use the default implementation provided by SecretFS.
/// 
/// # Example
/// 
/// ```rust
/// use secretfs::encryption::{SecretCipher, EncryptionError};
/// 
/// struct MyCustomCipher {
///     key: String,
/// }
/// 
/// impl SecretCipher for MyCustomCipher {
///     fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
///         // Your custom encryption logic here
///         Ok(plaintext.to_vec()) // Placeholder
///     }
///     
///     fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
///         // Your custom decryption logic here
///         Ok(ciphertext.to_vec()) // Placeholder
///     }
/// }
/// ```
pub trait SecretCipher: Send + Sync {
    /// Encrypt plaintext data
    /// 
    /// # Arguments
    /// * `plaintext` - The raw secret data to encrypt
    /// 
    /// # Returns
    /// * `Ok(Vec<u8>)` - Encrypted data
    /// * `Err(EncryptionError)` - If encryption fails
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError>;
    
    /// Decrypt ciphertext data
    /// 
    /// # Arguments
    /// * `ciphertext` - The encrypted data to decrypt
    /// 
    /// # Returns
    /// * `Ok(Vec<u8>)` - Decrypted plaintext data
    /// * `Err(EncryptionError)` - If decryption fails
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError>;
    
    /// Get cipher information for logging/debugging
    fn cipher_info(&self) -> String {
        "Custom SecretCipher".to_string()
    }
}

/// Default implementation using XOR cipher (for demo purposes)
/// 
/// ⚠️ WARNING: This is NOT cryptographically secure!
/// This is only for demonstration. In production, use proper encryption
/// like AES-256-GCM or implement your own secure cipher.
pub struct DefaultCipher {
    key: Vec<u8>,
}

impl DefaultCipher {
    /// Create a new DefaultCipher with a key
    /// 
    /// # Arguments
    /// * `key` - Encryption key (will be repeated if shorter than data)
    pub fn new(key: &str) -> Self {
        Self {
            key: key.as_bytes().to_vec(),
        }
    }
    
    /// Create DefaultCipher from environment variable or use default
    pub fn from_env() -> Self {
        let key = std::env::var("SECRETFS_ENCRYPTION_KEY")
            .unwrap_or_else(|_| "default-secretfs-key-2024".to_string());

        Self::new(&key)
    }
}

impl SecretCipher for DefaultCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if self.key.is_empty() {
            return Err(EncryptionError::InvalidKey("Key cannot be empty".to_string()));
        }
        
        let mut encrypted = Vec::with_capacity(plaintext.len());
        
        for (i, &byte) in plaintext.iter().enumerate() {
            let key_byte = self.key[i % self.key.len()];
            encrypted.push(byte ^ key_byte);
        }
        
        Ok(encrypted)
    }
    
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        if self.key.is_empty() {
            return Err(EncryptionError::InvalidKey("Key cannot be empty".to_string()));
        }
        
        // XOR is symmetric, so decryption is the same as encryption
        let mut decrypted = Vec::with_capacity(ciphertext.len());
        
        for (i, &byte) in ciphertext.iter().enumerate() {
            let key_byte = self.key[i % self.key.len()];
            decrypted.push(byte ^ key_byte);
        }
        
        Ok(decrypted)
    }
    
    fn cipher_info(&self) -> String {
        format!("DefaultCipher (XOR with {}-byte key) - ⚠️ DEMO ONLY, NOT SECURE!", self.key.len())
    }
}

/// No-op cipher that doesn't encrypt/decrypt (stores secrets in plaintext)
/// 
/// This is useful for development or when encryption is handled elsewhere.
pub struct PlaintextCipher;

impl PlaintextCipher {
    pub fn new() -> Self {
        Self
    }
}

impl SecretCipher for PlaintextCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        Ok(plaintext.to_vec())
    }
    
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        Ok(ciphertext.to_vec())
    }
    
    fn cipher_info(&self) -> String {
        "PlaintextCipher (no encryption) - ⚠️ SECRETS STORED IN PLAINTEXT!".to_string()
    }
}

/// RSA asymmetric cipher implementation
pub struct RsaCipher {
    encryption: AsymmetricEncryption,
}

impl RsaCipher {
    /// Create new RSA cipher with public key for encryption
    pub fn new() -> Result<Self, EncryptionError> {
        let encryption = AsymmetricEncryption::from_env()
            .map_err(|e| EncryptionError::InvalidKey(format!("RSA key error: {}", e)))?;

        Ok(RsaCipher { encryption })
    }
}

impl SecretCipher for RsaCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        self.encryption.encrypt(plaintext)
            .map_err(|e| EncryptionError::EncryptionFailed(format!("RSA encryption failed: {}", e)))
    }

    fn decrypt(&self, _ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // RSA cipher in SecretFS only encrypts - decryption happens in applications
        Err(EncryptionError::DecryptionFailed(
            "RSA decryption not available in SecretFS - use application with private key".to_string()
        ))
    }

    fn cipher_info(&self) -> String {
        format!("{} - ⚠️ SECRETS ENCRYPTED FOR AUTHORIZED APPLICATIONS ONLY!",
                self.encryption.encryption_info())
    }
}

/// Factory function to create cipher based on environment variable
/// 
/// Environment variable `SECRETFS_CIPHER_TYPE` can be:
/// - "default" or unset: Use DefaultCipher with XOR
/// - "plaintext": Use PlaintextCipher (no encryption)
/// - Custom implementations can be added here
pub fn create_cipher_from_env() -> Box<dyn SecretCipher> {
    let cipher_type = std::env::var("SECRETFS_CIPHER_TYPE")
        .unwrap_or_else(|_| "default".to_string())
        .to_lowercase();
    
    match cipher_type.as_str() {
        "plaintext" | "none" => {
            Box::new(PlaintextCipher::new())
        },
        "rsa" | "asymmetric" => {
            println!("🔐 RSA asymmetric encryption requested");
            match RsaCipher::new() {
                Ok(cipher) => {
                    println!("✅ RSA encryption initialized successfully");
                    println!("🔑 Only applications with private key can decrypt secrets");
                    println!("📋 RSA Configuration:");
                    println!("   • Cipher: {}", cipher.cipher_info());
                    println!("   • Security: Application-level access control");
                    Box::new(cipher)
                },
                Err(e) => {
                    eprintln!("❌ RSA encryption setup failed: {}", e);
                    eprintln!("💡 RSA requires public key configuration:");
                    eprintln!("   export SECRETFS_PUBLIC_KEY_FILE=/path/to/public.pem");
                    eprintln!("   # OR");
                    eprintln!("   export SECRETFS_PUBLIC_KEY_PEM=\"$(cat public.pem)\"");
                    eprintln!("📖 Generate keys with: ./target/release/secretfs-keygen generate private.pem public.pem");
                    eprintln!("🔄 Falling back to default symmetric encryption");
                    Box::new(DefaultCipher::from_env())
                }
            }
        },
        "default" | _ => {
            Box::new(DefaultCipher::from_env())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_cipher_encrypt_decrypt() {
        let cipher = DefaultCipher::new("test-key");
        let plaintext = b"hello world";
        
        let encrypted = cipher.encrypt(plaintext).unwrap();
        assert_ne!(encrypted, plaintext);
        
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_plaintext_cipher() {
        let cipher = PlaintextCipher::new();
        let plaintext = b"hello world";
        
        let encrypted = cipher.encrypt(plaintext).unwrap();
        assert_eq!(encrypted, plaintext);
        
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_empty_key_error() {
        let cipher = DefaultCipher::new("");
        let result = cipher.encrypt(b"test");
        assert!(result.is_err());
    }
}
