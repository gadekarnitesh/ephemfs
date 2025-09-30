use std::env;
use ephemfs::secret_client::convenience;

fn main() {
    println!("🔐 Rust Application - SecretFS RSA Decryption Demo");
    println!("==================================================");
    
    // Set up environment for demo
    let mount_path = env::var("SECRETFS_MOUNT_PATH")
        .unwrap_or_else(|_| "/tmp/secretfs_rsa_test".to_string());
    
    println!("📁 SecretFS Mount Path: {}", mount_path);
    
    // Check if private key is configured
    if env::var("SECRETFS_PRIVATE_KEY_FILE").is_err() && env::var("SECRETFS_PRIVATE_KEY_PEM").is_err() {
        println!("❌ No private key configured!");
        println!("   Set SECRETFS_PRIVATE_KEY_FILE or SECRETFS_PRIVATE_KEY_PEM");
        println!();
        println!("Example:");
        println!("   export SECRETFS_PRIVATE_KEY_FILE=/tmp/secretfs_keys/private.pem");
        println!("   cargo run --example rust_decrypt_demo");
        return;
    }
    
    println!("🔑 Private key configured - attempting to decrypt secrets...");
    println!();
    
    // Try to get all secrets
    match convenience::get_all_secrets() {
        Ok(secrets) => {
            if secrets.is_empty() {
                println!("📭 No secrets found in SecretFS");
            } else {
                println!("✅ Successfully decrypted {} secret(s):", secrets.len());
                println!();
                
                for (name, value) in secrets.iter() {
                    println!("🔓 Secret: {}", name);
                    println!("   Value: {}", value);
                    println!("   Length: {} characters", value.len());
                    println!();
                }
                
                // Demonstrate individual secret access
                println!("🔍 Individual Secret Access Examples:");
                println!();
                
                if let Ok(db_password) = convenience::get_secret("database_password") {
                    println!("✅ Database Password: {}", db_password);
                }
                
                if let Ok(api_key) = convenience::get_secret("api_key") {
                    println!("✅ API Key: {}", api_key);
                }
                
                if let Ok(jwt_secret) = convenience::get_secret("jwt_secret") {
                    println!("✅ JWT Secret: {}", jwt_secret);
                }
                
                println!();
                println!("🎉 All secrets successfully decrypted!");
                println!("   This demonstrates that only applications with the private key");
                println!("   can access the plaintext secrets, while 'cat' and other tools");
                println!("   only see encrypted binary data.");
            }
        },
        Err(e) => {
            println!("❌ Failed to decrypt secrets: {}", e);
            println!();
            println!("Possible causes:");
            println!("   • SecretFS not mounted at {}", mount_path);
            println!("   • Private key not matching the public key used for encryption");
            println!("   • Private key file not readable");
            println!("   • SecretFS using different encryption mode");
        }
    }
    
    println!();
    println!("🔐 Security Notes:");
    println!("   • This application has the private key and can decrypt secrets");
    println!("   • The private key should be kept secure and not shared");
    println!("   • Different applications can have different private keys");
    println!("   • SecretFS ensures secrets are never stored in plaintext on disk");
}
