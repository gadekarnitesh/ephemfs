use std::fs;
use std::path::Path;
use std::collections::HashMap;
use crate::asymmetric_encryption::AsymmetricDecryption;

/// Client for reading and decrypting secrets from SecretFS
pub struct SecretClient {
    decryption: Option<AsymmetricDecryption>,
    mount_path: String,
}

/// Error types for secret client operations
#[derive(Debug)]
pub enum SecretClientError {
    DecryptionError(String),
    FileError(String),
    ConfigurationError(String),
    NotFound(String),
}

impl std::fmt::Display for SecretClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SecretClientError::DecryptionError(msg) => write!(f, "Decryption error: {}", msg),
            SecretClientError::FileError(msg) => write!(f, "File error: {}", msg),
            SecretClientError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            SecretClientError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for SecretClientError {}

impl SecretClient {
    /// Create a new secret client with RSA decryption capability
    pub fn new_with_rsa_decryption(mount_path: &str) -> Result<Self, SecretClientError> {
        let decryption = AsymmetricDecryption::from_env()
            .map_err(|e| SecretClientError::ConfigurationError(format!("RSA decryption setup failed: {}", e)))?;
        
        Ok(SecretClient {
            decryption: Some(decryption),
            mount_path: mount_path.to_string(),
        })
    }
    
    /// Create a new secret client without decryption (for plaintext secrets)
    pub fn new_plaintext(mount_path: &str) -> Self {
        SecretClient {
            decryption: None,
            mount_path: mount_path.to_string(),
        }
    }
    
    /// Read and decrypt a secret by name
    pub fn get_secret(&self, secret_name: &str) -> Result<String, SecretClientError> {
        let secret_path = format!("{}/{}", self.mount_path, secret_name);
        
        if !Path::new(&secret_path).exists() {
            return Err(SecretClientError::NotFound(format!("Secret '{}' not found at {}", secret_name, secret_path)));
        }
        
        let encrypted_content = fs::read(&secret_path)
            .map_err(|e| SecretClientError::FileError(format!("Failed to read secret file {}: {}", secret_path, e)))?;
        
        if let Some(ref decryption) = self.decryption {
            // Decrypt the content
            let decrypted_bytes = decryption.decrypt(&encrypted_content)
                .map_err(|e| SecretClientError::DecryptionError(format!("Failed to decrypt secret '{}': {}", secret_name, e)))?;
            
            String::from_utf8(decrypted_bytes)
                .map_err(|e| SecretClientError::DecryptionError(format!("Decrypted content is not valid UTF-8: {}", e)))
        } else {
            // Return as plaintext
            String::from_utf8(encrypted_content)
                .map_err(|e| SecretClientError::FileError(format!("Secret content is not valid UTF-8: {}", e)))
        }
    }
    
    /// Read and decrypt a secret as bytes
    pub fn get_secret_bytes(&self, secret_name: &str) -> Result<Vec<u8>, SecretClientError> {
        let secret_path = format!("{}/{}", self.mount_path, secret_name);
        
        if !Path::new(&secret_path).exists() {
            return Err(SecretClientError::NotFound(format!("Secret '{}' not found at {}", secret_name, secret_path)));
        }
        
        let encrypted_content = fs::read(&secret_path)
            .map_err(|e| SecretClientError::FileError(format!("Failed to read secret file {}: {}", secret_path, e)))?;
        
        if let Some(ref decryption) = self.decryption {
            // Decrypt the content
            decryption.decrypt(&encrypted_content)
                .map_err(|e| SecretClientError::DecryptionError(format!("Failed to decrypt secret '{}': {}", secret_name, e)))
        } else {
            // Return as plaintext
            Ok(encrypted_content)
        }
    }
    
    /// List all available secrets
    pub fn list_secrets(&self) -> Result<Vec<String>, SecretClientError> {
        let mount_dir = Path::new(&self.mount_path);
        
        if !mount_dir.exists() {
            return Err(SecretClientError::FileError(format!("Mount path {} does not exist", self.mount_path)));
        }
        
        let entries = fs::read_dir(mount_dir)
            .map_err(|e| SecretClientError::FileError(format!("Failed to read mount directory {}: {}", self.mount_path, e)))?;
        
        let mut secrets = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| SecretClientError::FileError(format!("Failed to read directory entry: {}", e)))?;
            
            if entry.file_type().map_err(|e| SecretClientError::FileError(format!("Failed to get file type: {}", e)))?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    secrets.push(name.to_string());
                }
            }
        }
        
        secrets.sort();
        Ok(secrets)
    }
    
    /// Get all secrets as a HashMap
    pub fn get_all_secrets(&self) -> Result<HashMap<String, String>, SecretClientError> {
        let secret_names = self.list_secrets()?;
        let mut secrets = HashMap::new();
        
        for name in secret_names {
            match self.get_secret(&name) {
                Ok(value) => {
                    secrets.insert(name, value);
                },
                Err(e) => {
                    eprintln!("Warning: Failed to read secret '{}': {}", name, e);
                }
            }
        }
        
        Ok(secrets)
    }
    
    /// Check if the client has decryption capability
    pub fn has_decryption(&self) -> bool {
        self.decryption.is_some()
    }
    
    /// Get decryption info
    pub fn decryption_info(&self) -> String {
        if let Some(ref decryption) = self.decryption {
            decryption.decryption_info().to_string()
        } else {
            "No decryption - plaintext mode".to_string()
        }
    }
    
    /// Wait for a secret to become available (useful for initialization)
    pub fn wait_for_secret(&self, secret_name: &str, timeout_seconds: u64) -> Result<String, SecretClientError> {
        use std::time::{Duration, Instant};
        
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_seconds);
        
        loop {
            match self.get_secret(secret_name) {
                Ok(value) => return Ok(value),
                Err(SecretClientError::NotFound(_)) => {
                    if start.elapsed() > timeout {
                        return Err(SecretClientError::NotFound(format!(
                            "Secret '{}' not available after {} seconds", 
                            secret_name, 
                            timeout_seconds
                        )));
                    }
                    std::thread::sleep(Duration::from_millis(500));
                },
                Err(e) => return Err(e),
            }
        }
    }
}

/// Convenience functions for common use cases
pub mod convenience {
    use super::*;
    use std::env;
    
    /// Get a secret with automatic client configuration
    pub fn get_secret(secret_name: &str) -> Result<String, SecretClientError> {
        let mount_path = env::var("SECRETFS_MOUNT_PATH")
            .unwrap_or_else(|_| "/mnt/secrets".to_string());
        
        // Try RSA decryption first, fall back to plaintext
        match SecretClient::new_with_rsa_decryption(&mount_path) {
            Ok(client) => client.get_secret(secret_name),
            Err(_) => {
                let client = SecretClient::new_plaintext(&mount_path);
                client.get_secret(secret_name)
            }
        }
    }
    
    /// Get all secrets with automatic client configuration
    pub fn get_all_secrets() -> Result<HashMap<String, String>, SecretClientError> {
        let mount_path = env::var("SECRETFS_MOUNT_PATH")
            .unwrap_or_else(|_| "/mnt/secrets".to_string());
        
        // Try RSA decryption first, fall back to plaintext
        match SecretClient::new_with_rsa_decryption(&mount_path) {
            Ok(client) => client.get_all_secrets(),
            Err(_) => {
                let client = SecretClient::new_plaintext(&mount_path);
                client.get_all_secrets()
            }
        }
    }
    
    /// Wait for a secret with automatic client configuration
    pub fn wait_for_secret(secret_name: &str, timeout_seconds: u64) -> Result<String, SecretClientError> {
        let mount_path = env::var("SECRETFS_MOUNT_PATH")
            .unwrap_or_else(|_| "/mnt/secrets".to_string());
        
        // Try RSA decryption first, fall back to plaintext
        match SecretClient::new_with_rsa_decryption(&mount_path) {
            Ok(client) => client.wait_for_secret(secret_name, timeout_seconds),
            Err(_) => {
                let client = SecretClient::new_plaintext(&mount_path);
                client.wait_for_secret(secret_name, timeout_seconds)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_plaintext_client() {
        let temp_dir = TempDir::new().unwrap();
        let mount_path = temp_dir.path().to_str().unwrap();
        
        // Create a test secret file
        let secret_path = temp_dir.path().join("test_secret");
        fs::write(&secret_path, "test_value").unwrap();
        
        let client = SecretClient::new_plaintext(mount_path);
        
        // Test getting secret
        let value = client.get_secret("test_secret").unwrap();
        assert_eq!(value, "test_value");
        
        // Test listing secrets
        let secrets = client.list_secrets().unwrap();
        assert!(secrets.contains(&"test_secret".to_string()));
        
        // Test getting all secrets
        let all_secrets = client.get_all_secrets().unwrap();
        assert_eq!(all_secrets.get("test_secret"), Some(&"test_value".to_string()));
    }
    
    #[test]
    fn test_secret_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let mount_path = temp_dir.path().to_str().unwrap();
        
        let client = SecretClient::new_plaintext(mount_path);
        
        let result = client.get_secret("nonexistent");
        assert!(matches!(result, Err(SecretClientError::NotFound(_))));
    }
}
