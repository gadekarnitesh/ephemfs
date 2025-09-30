use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory,
    ReplyEntry, Request,
};
use libc::ENOENT;
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;

use std::time::{Duration, UNIX_EPOCH};

mod encryption;
mod secret_fetcher;
mod asymmetric_encryption;
pub mod secret_client;

use encryption::{SecretCipher, create_cipher_from_env};
use secret_fetcher::{SecretFetcher, SecretFetchConfig, create_fetcher_from_env};

const TTL: Duration = Duration::from_secs(1);

struct SecretFS {
    files: HashMap<u64, SecretFile>,
    paths: HashMap<String, u64>,
    next_inode: u64,
    cipher: Box<dyn SecretCipher>,
    fetcher: Box<dyn SecretFetcher>,
}

#[derive(Clone)]
struct SecretFile {
    inode: u64,
    name: String,
    content: Vec<u8>,  // ⚠️  SECURITY: Secrets stored ONLY in RAM, never written to disk
    attr: FileAttr,
    parent: u64,
    children: Vec<u64>,
}

// Security: Implement Drop to zero out memory when SecretFile is dropped
impl Drop for SecretFile {
    fn drop(&mut self) {
        // Zero out the secret content in memory for security
        // This prevents secrets from lingering in memory after use
        for byte in self.content.iter_mut() {
            *byte = 0;
        }
        // Also zero out the name to be extra secure
        unsafe {
            let name_bytes = self.name.as_bytes_mut();
            for byte in name_bytes.iter_mut() {
                *byte = 0;
            }
        }
    }
}

#[derive(Clone)]
struct Secret {
    name: String,
    content: String,
}

impl SecretFS {
    fn new() -> Self {
        // Create cipher based on environment configuration
        let cipher = create_cipher_from_env();

        // Create fetcher based on environment configuration
        let fetcher = create_fetcher_from_env();

        let mut fs = SecretFS {
            files: HashMap::new(),
            paths: HashMap::new(),
            next_inode: 2, // Start from 2, as 1 is reserved for root
            cipher,
            fetcher,
        };

        // Create root directory
        let root_attr = FileAttr {
            ino: 1,
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            flags: 0,
            blksize: 512,
        };

        let root_info = SecretFile {
            inode: 1,
            name: "/".to_string(),
            content: Vec::new(),
            attr: root_attr,
            parent: 1,
            children: Vec::new(),
        };

        fs.files.insert(1, root_info);
        fs.paths.insert("/".to_string(), 1);

        // Load secrets from environment or hardcoded values
        fs.load_secrets();

        // Show security information
        fs.security_info();

        fs
    }

    fn load_secrets(&mut self) {
        // First, try to load secrets from environment variables
        let env_secrets = self.get_secrets_from_env();

        // Then, try to fetch secrets from external URLs if configured
        let fetched_secrets = self.fetch_external_secrets();

        // Combine both sources
        let mut all_secrets = env_secrets;
        all_secrets.extend(fetched_secrets);

        if all_secrets.is_empty() {
            println!("⚠️  No secrets configured");
        }

        for secret in all_secrets {
            self.add_secret_file(&secret);
        }
    }

    fn fetch_external_secrets(&self) -> Vec<Secret> {
        // Check if external fetching is configured
        if env::var("SECRETFS_URLS").is_err() {
            return Vec::new();
        }

        // Try to create fetch configuration
        let config = match SecretFetchConfig::from_env() {
            Ok(config) => {
                if let Err(e) = config.validate() {
                    eprintln!("❌ Invalid secret fetch configuration: {}", e);
                    return Vec::new();
                }
                config
            },
            Err(e) => {
                eprintln!("❌ Failed to create secret fetch configuration: {}", e);
                return Vec::new();
            }
        };

        // Fetch secrets using the configured fetcher
        match self.fetcher.fetch_secrets(&config) {
            Ok(fetched_secrets) => {
                // Convert FetchedSecret to Secret
                fetched_secrets.into_iter().map(|fs| Secret {
                    name: fs.key,
                    content: fs.value,
                }).collect()
            },
            Err(e) => {
                eprintln!("❌ Failed to fetch external secrets: {}", e);
                Vec::new()
            }
        }
    }

    fn get_secrets_from_env(&self) -> Vec<Secret> {
        let mut secrets = Vec::new();

        // Check for individual secret environment variables
        if let Ok(db_pass) = env::var("DATABASE_PASSWORD") {
            secrets.push(Secret {
                name: "database_password".to_string(),
                content: db_pass,
            });
        }

        if let Ok(api_key) = env::var("API_KEY") {
            secrets.push(Secret {
                name: "api_key".to_string(),
                content: api_key,
            });
        }

        if let Ok(jwt_secret) = env::var("JWT_SECRET") {
            secrets.push(Secret {
                name: "jwt_secret".to_string(),
                content: jwt_secret,
            });
        }

        if let Ok(redis_pass) = env::var("REDIS_PASSWORD") {
            secrets.push(Secret {
                name: "redis_password".to_string(),
                content: redis_pass,
            });
        }

        if let Ok(vault_token) = env::var("VAULT_TOKEN") {
            secrets.push(Secret {
                name: "vault_token".to_string(),
                content: vault_token,
            });
        }

        // Check for config file content
        if let Ok(config_content) = env::var("CONFIG_JSON") {
            secrets.push(Secret {
                name: "config.json".to_string(),
                content: config_content,
            });
        }

        // Check for custom secrets via SECRET_* pattern
        for (key, value) in env::vars() {
            if key.starts_with("SECRET_") {
                let secret_name = key.strip_prefix("SECRET_")
                    .unwrap()
                    .to_lowercase()
                    .replace('_', "-");
                secrets.push(Secret {
                    name: secret_name,
                    content: value,
                });
            }
        }



        secrets
    }

    fn add_secret_file(&mut self, secret: &Secret) {
        let inode = self.next_inode;
        self.next_inode += 1;

        // Encrypt the secret content before storing
        let plaintext_bytes = secret.content.as_bytes();
        let encrypted_content = match self.cipher.encrypt(plaintext_bytes) {
            Ok(encrypted) => encrypted,
            Err(e) => {
                eprintln!("❌ Failed to encrypt secret '{}': {}", secret.name, e);
                return;
            }
        };

        let size = encrypted_content.len() as u64;

        let attr = FileAttr {
            ino: inode,
            size,
            blocks: (size + 511) / 512,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::RegularFile,
            perm: 0o600, // Read-only for owner only (secure)
            nlink: 1,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            flags: 0,
            blksize: 512,
        };

        let secret_file = SecretFile {
            inode,
            name: secret.name.clone(),
            content: encrypted_content,
            attr,
            parent: 1, // All secrets are in root directory
            children: Vec::new(),
        };

        self.files.insert(inode, secret_file);
        self.paths.insert(format!("/{}", secret.name), inode);

        // Add to root directory's children
        if let Some(root) = self.files.get_mut(&1) {
            root.children.push(inode);
        }
    }

    /// Security: Demonstrate that secrets exist only in memory
    fn security_info(&self) {
        let total_secrets = self.files.len() - 1; // Exclude root directory

        println!("✅ Loaded {} secret(s) | Encryption: {} | Memory-only storage",
                 total_secrets,
                 self.cipher.cipher_info());
    }
}

impl Filesystem for SecretFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {

        if let Some(parent_info) = self.files.get(&parent) {
            for &child_inode in &parent_info.children {
                if let Some(child_info) = self.files.get(&child_inode) {
                    if child_info.name == name.to_string_lossy() {
                        reply.entry(&TTL, &child_info.attr, 0);
                        return;
                    }
                }
            }
        }
        reply.error(ENOENT);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {

        if let Some(file_info) = self.files.get(&ino) {
            reply.attr(&TTL, &file_info.attr);
        } else {
            reply.error(ENOENT);
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {

        if let Some(file_info) = self.files.get(&ino) {
            if file_info.attr.kind == FileType::RegularFile {
                // Check if this is RSA encryption (which doesn't support decryption in SecretFS)
                let cipher_info = self.cipher.cipher_info();
                if cipher_info.contains("RSA") || cipher_info.contains("AUTHORIZED APPLICATIONS ONLY") {
                    // For RSA encryption, serve the encrypted data directly
                    // Applications with private keys will decrypt it themselves
                    let start = offset as usize;
                    let end = std::cmp::min(start + size as usize, file_info.content.len());
                    if start < file_info.content.len() {
                        reply.data(&file_info.content[start..end]);
                    } else {
                        reply.data(&[]);
                    }
                } else {
                    // For other ciphers, decrypt the content before serving it
                    let decrypted_data = match self.cipher.decrypt(&file_info.content) {
                        Ok(data) => data,
                        Err(e) => {
                            eprintln!("❌ Failed to decrypt secret '{}': {}", file_info.name, e);
                            reply.error(libc::EIO); // I/O error
                            return;
                        }
                    };

                    let start = offset as usize;
                    let end = std::cmp::min(start + size as usize, decrypted_data.len());
                    if start < decrypted_data.len() {
                        reply.data(&decrypted_data[start..end]);
                    } else {
                        reply.data(&[]);
                    }
                }
            } else {
                reply.error(ENOENT);
            }
        } else {
            reply.error(ENOENT);
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {

        if let Some(dir_info) = self.files.get(&ino) {
            if dir_info.attr.kind != FileType::Directory {
                reply.error(ENOENT);
                return;
            }

            let mut entries = vec![
                (1, FileType::Directory, "."),
                (dir_info.parent, FileType::Directory, ".."),
            ];

            for &child_inode in &dir_info.children {
                if let Some(child_info) = self.files.get(&child_inode) {
                    entries.push((child_inode, child_info.attr.kind, &child_info.name));
                }
            }

            for (i, entry) in entries.iter().enumerate().skip(offset as usize) {
                if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                    break;
                }
            }
            reply.ok();
        } else {
            reply.error(ENOENT);
        }
    }

    // Security: Explicitly prevent write operations to maintain memory-only guarantee
    fn write(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        _data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyWrite,
    ) {
        println!("🚫 SECURITY: Write operation blocked - SecretFS is memory-only and read-only");
        reply.error(libc::EROFS); // Read-only filesystem error
    }

    fn create(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        _umask: u32,
        _flags: i32,
        reply: fuser::ReplyCreate,
    ) {
        println!("🚫 SECURITY: Create operation blocked - SecretFS is memory-only and read-only");
        reply.error(libc::EROFS); // Read-only filesystem error
    }

    fn unlink(&mut self, _req: &Request, _parent: u64, _name: &OsStr, reply: fuser::ReplyEmpty) {
        println!("🚫 SECURITY: Unlink operation blocked - SecretFS is memory-only and read-only");
        reply.error(libc::EROFS); // Read-only filesystem error
    }

    fn mkdir(
        &mut self,
        _req: &Request,
        _parent: u64,
        _name: &OsStr,
        _mode: u32,
        _umask: u32,
        reply: fuser::ReplyEntry,
    ) {
        println!("🚫 SECURITY: Mkdir operation blocked - SecretFS is memory-only and read-only");
        reply.error(libc::EROFS); // Read-only filesystem error
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <mount_point>", args[0]);
        eprintln!("Example: {} /mnt/secrets", args[0]);
        eprintln!("");
        eprintln!("Environment variables:");
        eprintln!("  FUSE_MOUNTPOINT        - Mount point (optional, overrides command line)");
        eprintln!("  DATABASE_PASSWORD      - Database password secret");
        eprintln!("  API_KEY                - API key secret");
        eprintln!("  JWT_SECRET             - JWT signing secret");
        eprintln!("  SECRET_<NAME>          - Custom secrets (e.g., SECRET_STRIPE_KEY)");
        eprintln!("");
        eprintln!("Encryption configuration:");
        eprintln!("  SECRETFS_CIPHER_TYPE   - Encryption method:");
        eprintln!("                           • 'default' - XOR cipher (demo/development)");
        eprintln!("                           • 'plaintext' - No encryption");
        eprintln!("                           • 'rsa' - RSA asymmetric encryption (production)");
        eprintln!("  SECRETFS_ENCRYPTION_KEY - Encryption key (for default cipher)");
        eprintln!("");
        eprintln!("RSA encryption configuration (when SECRETFS_CIPHER_TYPE=rsa):");
        eprintln!("  SECRETFS_PUBLIC_KEY_FILE - Path to RSA public key file");
        eprintln!("  SECRETFS_PUBLIC_KEY_PEM  - RSA public key in PEM format");
        eprintln!("  Generate keys with: ./target/release/secretfs-keygen generate private.pem public.pem");
        eprintln!("");
        eprintln!("External secret fetching:");
        eprintln!("  SECRETFS_URLS          - Comma-separated URLs to fetch secrets from");
        eprintln!("  SECRETFS_AUTH_TOKEN    - Bearer token for API authentication");
        eprintln!("  SECRETFS_FETCHER_TYPE  - 'http' (default) or 'mock' (for testing)");
        eprintln!("  SECRETFS_TIMEOUT_SECONDS - HTTP timeout in seconds (default: 30)");
        eprintln!("  SECRETFS_RETRY_ATTEMPTS - Number of retry attempts (default: 3)");
        eprintln!("  SECRETFS_HEADERS       - Custom headers (format: 'Key1:Value1,Key2:Value2')");
        std::process::exit(1);
    }

    let mount_point = env::var("FUSE_MOUNTPOINT").unwrap_or_else(|_| args[1].clone());

    println!("🔒 SecretFS mounted at: {}", mount_point);

    let filesystem = SecretFS::new();

    println!("Press Ctrl+C to unmount\n");

    let options = vec![
        MountOption::RO,
        MountOption::FSName("secretfs".to_string()),
    ];

    match fuser::mount2(filesystem, &mount_point, &options) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("❌ Failed to mount: {}", e);
            std::process::exit(1);
        }
    }
}
