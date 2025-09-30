use std::env;
use std::path::Path;
use ephemfs::asymmetric_encryption::key_utils;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    match args[1].as_str() {
        "generate" => {
            if args.len() < 4 {
                eprintln!("Usage: secretfs-keygen generate <private_key_file> <public_key_file> [key_size]");
                return;
            }
            
            let private_key_file = &args[2];
            let public_key_file = &args[3];
            let key_size = if args.len() > 4 {
                args[4].parse().unwrap_or(2048)
            } else {
                2048
            };
            
            generate_keys(private_key_file, public_key_file, key_size);
        },
        "info" => {
            if args.len() < 3 {
                eprintln!("Usage: secretfs-keygen info <key_file>");
                return;
            }
            
            let key_file = &args[2];
            show_key_info(key_file);
        },
        "help" | "--help" | "-h" => {
            print_usage();
        },
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("SecretFS Key Generation Utility");
    println!("===============================");
    println!();
    println!("USAGE:");
    println!("  secretfs-keygen generate <private_key_file> <public_key_file> [key_size]");
    println!("  secretfs-keygen info <key_file>");
    println!("  secretfs-keygen help");
    println!();
    println!("COMMANDS:");
    println!("  generate    Generate a new RSA key pair");
    println!("  info        Display information about a key file");
    println!("  help        Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("  # Generate 2048-bit RSA key pair");
    println!("  secretfs-keygen generate private.pem public.pem");
    println!();
    println!("  # Generate 4096-bit RSA key pair");
    println!("  secretfs-keygen generate private.pem public.pem 4096");
    println!();
    println!("  # Show key information");
    println!("  secretfs-keygen info public.pem");
    println!();
    println!("SECURITY NOTES:");
    println!("  • Keep private keys secure and never share them");
    println!("  • Distribute only public keys for encryption");
    println!("  • Use 2048-bit keys minimum, 4096-bit for high security");
    println!("  • Store private keys in secure locations with proper permissions");
}

fn generate_keys(private_key_file: &str, public_key_file: &str, key_size: usize) {
    // Validate key size
    if key_size < 1024 {
        eprintln!("❌ Error: Key size must be at least 1024 bits");
        return;
    }
    
    if key_size > 8192 {
        eprintln!("❌ Error: Key size cannot exceed 8192 bits");
        return;
    }
    
    // Check if files already exist
    if Path::new(private_key_file).exists() {
        eprintln!("❌ Error: Private key file '{}' already exists", private_key_file);
        eprintln!("   Remove the existing file or choose a different name");
        return;
    }
    
    if Path::new(public_key_file).exists() {
        eprintln!("❌ Error: Public key file '{}' already exists", public_key_file);
        eprintln!("   Remove the existing file or choose a different name");
        return;
    }
    
    println!("🔑 Generating RSA-{} key pair...", key_size);
    println!("   Private key: {}", private_key_file);
    println!("   Public key: {}", public_key_file);
    println!();
    
    match key_utils::generate_key_pair(key_size, private_key_file, public_key_file) {
        Ok(()) => {
            println!();
            println!("✅ Key pair generated successfully!");
            println!();
            println!("📋 NEXT STEPS:");
            println!("   1. Secure the private key:");
            println!("      chmod 600 {}", private_key_file);
            println!("      # Move to secure location if needed");
            println!();
            println!("   2. Configure SecretFS with the public key:");
            println!("      export SECRETFS_CIPHER_TYPE=rsa");
            println!("      export SECRETFS_PUBLIC_KEY_FILE={}", public_key_file);
            println!("      # OR");
            println!("      export SECRETFS_PUBLIC_KEY_PEM=\"$(cat {})\"", public_key_file);
            println!();
            println!("   3. Configure applications with the private key:");
            println!("      export SECRETFS_PRIVATE_KEY_FILE={}", private_key_file);
            println!("      # OR");
            println!("      export SECRETFS_PRIVATE_KEY_PEM=\"$(cat {})\"", private_key_file);
            println!();
            println!("🔐 SECURITY REMINDERS:");
            println!("   • Private key can decrypt all secrets - keep it secure!");
            println!("   • Public key is safe to distribute and store in containers");
            println!("   • Use different key pairs for different environments");
            println!("   • Regularly rotate keys according to your security policy");
        },
        Err(e) => {
            eprintln!("❌ Failed to generate key pair: {}", e);
        }
    }
}

fn show_key_info(key_file: &str) {
    if !Path::new(key_file).exists() {
        eprintln!("❌ Error: Key file '{}' does not exist", key_file);
        return;
    }
    
    match key_utils::display_key_info(key_file) {
        Ok(()) => {
            println!();
            if key_file.contains("private") || key_file.contains("priv") {
                println!("🔐 PRIVATE KEY SECURITY:");
                println!("   • This key can decrypt secrets - keep it secure!");
                println!("   • Set restrictive permissions: chmod 600 {}", key_file);
                println!("   • Store in secure location (e.g., /etc/secretfs/keys/)");
                println!("   • Never commit to version control");
                println!("   • Use secure key management systems in production");
            } else {
                println!("🔓 PUBLIC KEY USAGE:");
                println!("   • This key is safe to distribute");
                println!("   • Use for SecretFS encryption configuration");
                println!("   • Can be stored in container images");
                println!("   • Share with teams that need to encrypt secrets");
            }
        },
        Err(e) => {
            eprintln!("❌ Failed to read key file: {}", e);
        }
    }
}
