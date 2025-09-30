use std::env;
use std::process;
use ephemfs::secret_client::{SecretClient, SecretClientError};

fn main() {
    println!("🔐 SecretFS Client Application Example");
    println!("=====================================");
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "get" => {
            if args.len() < 3 {
                eprintln!("Usage: secret_reader_app get <secret_name>");
                process::exit(1);
            }
            get_secret(&args[2]);
        },
        "list" => {
            list_secrets();
        },
        "all" => {
            get_all_secrets();
        },
        "wait" => {
            if args.len() < 3 {
                eprintln!("Usage: secret_reader_app wait <secret_name> [timeout_seconds]");
                process::exit(1);
            }
            let timeout = if args.len() > 3 {
                args[3].parse().unwrap_or(30)
            } else {
                30
            };
            wait_for_secret(&args[2], timeout);
        },
        "info" => {
            show_client_info();
        },
        "help" | "--help" | "-h" => {
            print_usage();
        },
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    println!();
    println!("USAGE:");
    println!("  secret_reader_app get <secret_name>           Get a specific secret");
    println!("  secret_reader_app list                        List all available secrets");
    println!("  secret_reader_app all                         Get all secrets");
    println!("  secret_reader_app wait <secret_name> [timeout] Wait for a secret to become available");
    println!("  secret_reader_app info                        Show client configuration info");
    println!("  secret_reader_app help                        Show this help message");
    println!();
    println!("ENVIRONMENT VARIABLES:");
    println!("  SECRETFS_MOUNT_PATH                           Path to SecretFS mount (default: /mnt/secrets)");
    println!("  SECRETFS_PRIVATE_KEY_FILE                     Path to RSA private key file");
    println!("  SECRETFS_PRIVATE_KEY_PEM                      RSA private key in PEM format");
    println!();
    println!("EXAMPLES:");
    println!("  # Get database password");
    println!("  secret_reader_app get database_password");
    println!();
    println!("  # List all available secrets");
    println!("  secret_reader_app list");
    println!();
    println!("  # Get all secrets at once");
    println!("  secret_reader_app all");
    println!();
    println!("  # Wait for a secret to become available (useful for initialization)");
    println!("  secret_reader_app wait api_key 60");
    println!();
    println!("  # Show client configuration");
    println!("  secret_reader_app info");
    println!();
    println!("SECURITY NOTES:");
    println!("  • This application requires the RSA private key to decrypt secrets");
    println!("  • Without the private key, it will fall back to plaintext mode");
    println!("  • Keep private keys secure and use proper access controls");
    println!("  • Use different key pairs for different environments");
}

fn get_secret(secret_name: &str) {
    println!("🔍 Getting secret: {}", secret_name);
    
    match create_client() {
        Ok(client) => {
            match client.get_secret(secret_name) {
                Ok(value) => {
                    println!("✅ Secret retrieved successfully:");
                    println!("   Name: {}", secret_name);
                    println!("   Value: {}", value);
                    println!("   Length: {} characters", value.len());
                },
                Err(e) => {
                    eprintln!("❌ Failed to get secret '{}': {}", secret_name, e);
                    process::exit(1);
                }
            }
        },
        Err(e) => {
            eprintln!("❌ Failed to create client: {}", e);
            process::exit(1);
        }
    }
}

fn list_secrets() {
    println!("📋 Listing all available secrets...");
    
    match create_client() {
        Ok(client) => {
            match client.list_secrets() {
                Ok(secrets) => {
                    if secrets.is_empty() {
                        println!("   No secrets found");
                    } else {
                        println!("   Found {} secret(s):", secrets.len());
                        for (i, secret) in secrets.iter().enumerate() {
                            println!("   {}. {}", i + 1, secret);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("❌ Failed to list secrets: {}", e);
                    process::exit(1);
                }
            }
        },
        Err(e) => {
            eprintln!("❌ Failed to create client: {}", e);
            process::exit(1);
        }
    }
}

fn get_all_secrets() {
    println!("📦 Getting all secrets...");
    
    match create_client() {
        Ok(client) => {
            match client.get_all_secrets() {
                Ok(secrets) => {
                    if secrets.is_empty() {
                        println!("   No secrets found");
                    } else {
                        println!("   Retrieved {} secret(s):", secrets.len());
                        for (name, value) in secrets.iter() {
                            println!("   • {}: {} ({} chars)", name, value, value.len());
                        }
                    }
                },
                Err(e) => {
                    eprintln!("❌ Failed to get all secrets: {}", e);
                    process::exit(1);
                }
            }
        },
        Err(e) => {
            eprintln!("❌ Failed to create client: {}", e);
            process::exit(1);
        }
    }
}

fn wait_for_secret(secret_name: &str, timeout_seconds: u64) {
    println!("⏳ Waiting for secret '{}' (timeout: {} seconds)...", secret_name, timeout_seconds);
    
    match create_client() {
        Ok(client) => {
            match client.wait_for_secret(secret_name, timeout_seconds) {
                Ok(value) => {
                    println!("✅ Secret became available:");
                    println!("   Name: {}", secret_name);
                    println!("   Value: {}", value);
                    println!("   Length: {} characters", value.len());
                },
                Err(e) => {
                    eprintln!("❌ Failed to wait for secret '{}': {}", secret_name, e);
                    process::exit(1);
                }
            }
        },
        Err(e) => {
            eprintln!("❌ Failed to create client: {}", e);
            process::exit(1);
        }
    }
}

fn show_client_info() {
    println!("ℹ️  SecretFS Client Configuration:");
    
    let mount_path = env::var("SECRETFS_MOUNT_PATH")
        .unwrap_or_else(|_| "/mnt/secrets".to_string());
    
    println!("   Mount Path: {}", mount_path);
    
    // Check if mount path exists
    if std::path::Path::new(&mount_path).exists() {
        println!("   Mount Status: ✅ Available");
    } else {
        println!("   Mount Status: ❌ Not found");
    }
    
    // Check RSA configuration
    if env::var("SECRETFS_PRIVATE_KEY_FILE").is_ok() {
        let key_file = env::var("SECRETFS_PRIVATE_KEY_FILE").unwrap();
        println!("   Private Key File: {}", key_file);
        if std::path::Path::new(&key_file).exists() {
            println!("   Private Key Status: ✅ Available");
        } else {
            println!("   Private Key Status: ❌ File not found");
        }
    } else if env::var("SECRETFS_PRIVATE_KEY_PEM").is_ok() {
        println!("   Private Key: ✅ Provided via environment variable");
    } else {
        println!("   Private Key: ❌ Not configured");
        println!("   Decryption Mode: Plaintext fallback");
    }
    
    // Try to create client and show its info
    match create_client() {
        Ok(client) => {
            println!("   Client Status: ✅ Ready");
            println!("   Decryption Capability: {}", if client.has_decryption() { "✅ Available" } else { "❌ Plaintext only" });
            println!("   Decryption Info: {}", client.decryption_info());
        },
        Err(e) => {
            println!("   Client Status: ❌ Failed to initialize");
            println!("   Error: {}", e);
        }
    }
}

fn create_client() -> Result<SecretClient, SecretClientError> {
    let mount_path = env::var("SECRETFS_MOUNT_PATH")
        .unwrap_or_else(|_| "/mnt/secrets".to_string());
    
    // Try RSA decryption first, fall back to plaintext
    match SecretClient::new_with_rsa_decryption(&mount_path) {
        Ok(client) => {
            println!("🔐 Using RSA decryption mode");
            Ok(client)
        },
        Err(_) => {
            println!("⚠️  RSA decryption not available, using plaintext mode");
            Ok(SecretClient::new_plaintext(&mount_path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_client_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mount_path = temp_dir.path().to_str().unwrap();
        
        // Test plaintext client creation
        let client = SecretClient::new_plaintext(mount_path);
        assert!(!client.has_decryption());
        assert_eq!(client.decryption_info(), "No decryption - plaintext mode");
    }
    
    #[test]
    fn test_secret_operations() {
        let temp_dir = TempDir::new().unwrap();
        let mount_path = temp_dir.path().to_str().unwrap();
        
        // Create test secrets
        fs::write(temp_dir.path().join("test_secret1"), "value1").unwrap();
        fs::write(temp_dir.path().join("test_secret2"), "value2").unwrap();
        
        let client = SecretClient::new_plaintext(mount_path);
        
        // Test get secret
        let value = client.get_secret("test_secret1").unwrap();
        assert_eq!(value, "value1");
        
        // Test list secrets
        let secrets = client.list_secrets().unwrap();
        assert_eq!(secrets.len(), 2);
        assert!(secrets.contains(&"test_secret1".to_string()));
        assert!(secrets.contains(&"test_secret2".to_string()));
        
        // Test get all secrets
        let all_secrets = client.get_all_secrets().unwrap();
        assert_eq!(all_secrets.len(), 2);
        assert_eq!(all_secrets.get("test_secret1"), Some(&"value1".to_string()));
        assert_eq!(all_secrets.get("test_secret2"), Some(&"value2".to_string()));
    }
}
