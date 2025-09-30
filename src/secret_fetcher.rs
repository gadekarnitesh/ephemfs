use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt;
use std::time::Duration;

/// Custom error type for secret fetching operations
#[derive(Debug)]
pub enum SecretFetchError {
    NetworkError(String),
    AuthenticationError(String),
    ParseError(String),
    ConfigurationError(String),
    TimeoutError(String),
}

impl fmt::Display for SecretFetchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SecretFetchError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            SecretFetchError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            SecretFetchError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SecretFetchError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            SecretFetchError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
        }
    }
}

impl Error for SecretFetchError {}

/// Represents a secret fetched from an external source
#[derive(Debug, Clone)]
pub struct FetchedSecret {
    pub key: String,
    pub value: String,
    pub source_url: String,
    pub metadata: HashMap<String, String>,
}

/// Configuration for secret fetching
#[derive(Debug, Clone)]
pub struct SecretFetchConfig {
    pub urls: Vec<String>,
    pub auth_token: Option<String>,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub user_agent: String,
    pub headers: HashMap<String, String>,
}

impl SecretFetchConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, SecretFetchError> {
        let urls_str = env::var("SECRETFS_URLS")
            .map_err(|_| SecretFetchError::ConfigurationError(
                "SECRETFS_URLS environment variable not set".to_string()
            ))?;
        
        let urls: Vec<String> = urls_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        if urls.is_empty() {
            return Err(SecretFetchError::ConfigurationError(
                "No valid URLs found in SECRETFS_URLS".to_string()
            ));
        }
        
        let auth_token = env::var("SECRETFS_AUTH_TOKEN").ok();
        
        let timeout_seconds = env::var("SECRETFS_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);
        
        let retry_attempts = env::var("SECRETFS_RETRY_ATTEMPTS")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);
        
        let user_agent = env::var("SECRETFS_USER_AGENT")
            .unwrap_or_else(|_| "SecretFS/1.0".to_string());
        
        // Parse additional headers from environment
        let mut headers = HashMap::new();
        
        // Add common headers
        if let Some(ref token) = auth_token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }
        
        // Parse custom headers from SECRETFS_HEADERS (format: "Key1:Value1,Key2:Value2")
        if let Ok(headers_str) = env::var("SECRETFS_HEADERS") {
            for header_pair in headers_str.split(',') {
                if let Some((key, value)) = header_pair.split_once(':') {
                    headers.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
        
        Ok(SecretFetchConfig {
            urls,
            auth_token,
            timeout_seconds,
            retry_attempts,
            user_agent,
            headers,
        })
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), SecretFetchError> {
        if self.urls.is_empty() {
            return Err(SecretFetchError::ConfigurationError(
                "No URLs configured".to_string()
            ));
        }
        
        for url in &self.urls {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(SecretFetchError::ConfigurationError(
                    format!("Invalid URL format: {}", url)
                ));
            }
        }
        
        if self.timeout_seconds == 0 {
            return Err(SecretFetchError::ConfigurationError(
                "Timeout must be greater than 0".to_string()
            ));
        }
        
        Ok(())
    }
}

/// Trait for different secret fetching strategies
pub trait SecretFetcher: Send + Sync {
    /// Fetch secrets from the configured sources
    fn fetch_secrets(&self, config: &SecretFetchConfig) -> Result<Vec<FetchedSecret>, SecretFetchError>;
    
    /// Get fetcher information for logging
    fn fetcher_info(&self) -> String {
        "Generic SecretFetcher".to_string()
    }
}

/// HTTP-based secret fetcher
pub struct HttpSecretFetcher {
    client: Option<reqwest::Client>,
}

impl HttpSecretFetcher {
    pub fn new() -> Self {
        // Create HTTP client with reasonable defaults
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .ok();
        
        Self { client }
    }
    
    /// Parse JSON response into secrets
    fn parse_json_secrets(&self, json_str: &str, source_url: &str) -> Result<Vec<FetchedSecret>, SecretFetchError> {
        let json_value: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| SecretFetchError::ParseError(format!("Invalid JSON: {}", e)))?;
        
        let mut secrets = Vec::new();
        
        match json_value {
            // Handle flat key-value object: {"key1": "value1", "key2": "value2"}
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    if let serde_json::Value::String(string_value) = value {
                        secrets.push(FetchedSecret {
                            key: key.clone(),
                            value: string_value,
                            source_url: source_url.to_string(),
                            metadata: HashMap::new(),
                        });
                    } else {
                        // Convert non-string values to JSON strings
                        secrets.push(FetchedSecret {
                            key: key.clone(),
                            value: value.to_string(),
                            source_url: source_url.to_string(),
                            metadata: HashMap::new(),
                        });
                    }
                }
            },
            // Handle array of secret objects: [{"key": "name", "value": "secret"}, ...]
            serde_json::Value::Array(arr) => {
                for item in arr {
                    if let serde_json::Value::Object(obj) = item {
                        let key = obj.get("key")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| SecretFetchError::ParseError("Missing 'key' field".to_string()))?
                            .to_string();

                        let value = obj.get("value")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| SecretFetchError::ParseError("Missing 'value' field".to_string()))?
                            .to_string();

                        let mut metadata = HashMap::new();
                        for (meta_key, meta_value) in &obj {
                            if meta_key != "key" && meta_key != "value" {
                                if let Some(meta_str) = meta_value.as_str() {
                                    metadata.insert(meta_key.clone(), meta_str.to_string());
                                }
                            }
                        }

                        secrets.push(FetchedSecret {
                            key,
                            value,
                            source_url: source_url.to_string(),
                            metadata,
                        });
                    }
                }
            },
            _ => {
                return Err(SecretFetchError::ParseError(
                    "JSON must be an object or array".to_string()
                ));
            }
        }
        
        Ok(secrets)
    }
    
    /// Fetch secrets from a single URL
    async fn fetch_from_url(&self, url: &str, config: &SecretFetchConfig) -> Result<Vec<FetchedSecret>, SecretFetchError> {
        let client = self.client.as_ref()
            .ok_or_else(|| SecretFetchError::NetworkError("HTTP client not available".to_string()))?;
        
        let mut request = client
            .get(url)
            .timeout(Duration::from_secs(config.timeout_seconds))
            .header("User-Agent", &config.user_agent);
        
        // Add custom headers
        for (key, value) in &config.headers {
            request = request.header(key, value);
        }
        
        let response = request
            .send()
            .await
            .map_err(|e| SecretFetchError::NetworkError(format!("Request failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(SecretFetchError::NetworkError(
                format!("HTTP {} from {}", response.status(), url)
            ));
        }
        
        let body = response
            .text()
            .await
            .map_err(|e| SecretFetchError::NetworkError(format!("Failed to read response: {}", e)))?;
        
        self.parse_json_secrets(&body, url)
    }
}

impl SecretFetcher for HttpSecretFetcher {
    fn fetch_secrets(&self, config: &SecretFetchConfig) -> Result<Vec<FetchedSecret>, SecretFetchError> {
        // For now, return a placeholder implementation
        // In a real implementation, you would use async/await with tokio
        println!("🌐 Fetching secrets from {} URLs", config.urls.len());
        
        for url in &config.urls {
            println!("   📡 URL: {}", url);
        }
        
        // Placeholder: return empty list for now
        // TODO: Implement actual HTTP fetching with tokio runtime
        Ok(Vec::new())
    }
    
    fn fetcher_info(&self) -> String {
        "HttpSecretFetcher (HTTP/HTTPS JSON API)".to_string()
    }
}

/// Mock fetcher for testing and development
pub struct MockSecretFetcher {
    mock_secrets: Vec<FetchedSecret>,
}

impl MockSecretFetcher {
    pub fn new() -> Self {
        let mock_secrets = vec![
            FetchedSecret {
                key: "mock_api_key".to_string(),
                value: "mock-api-key-12345".to_string(),
                source_url: "mock://test".to_string(),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("type".to_string(), "api_key".to_string());
                    map.insert("environment".to_string(), "test".to_string());
                    map
                },
            },
            FetchedSecret {
                key: "mock_database_url".to_string(),
                value: "postgresql://user:pass@localhost:5432/testdb".to_string(),
                source_url: "mock://test".to_string(),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("type".to_string(), "database_url".to_string());
                    map.insert("environment".to_string(), "test".to_string());
                    map
                },
            },
        ];
        
        Self { mock_secrets }
    }
}

impl SecretFetcher for MockSecretFetcher {
    fn fetch_secrets(&self, config: &SecretFetchConfig) -> Result<Vec<FetchedSecret>, SecretFetchError> {
        println!("🧪 Mock fetcher: simulating fetch from {} URLs", config.urls.len());
        
        // Simulate some processing time
        std::thread::sleep(Duration::from_millis(100));
        
        Ok(self.mock_secrets.clone())
    }
    
    fn fetcher_info(&self) -> String {
        "MockSecretFetcher (for testing and development)".to_string()
    }
}

/// Factory function to create fetcher based on environment configuration
pub fn create_fetcher_from_env() -> Box<dyn SecretFetcher> {
    let fetcher_type = env::var("SECRETFS_FETCHER_TYPE")
        .unwrap_or_else(|_| "http".to_string())
        .to_lowercase();
    
    match fetcher_type.as_str() {
        "mock" | "test" => {
            Box::new(MockSecretFetcher::new())
        },
        "http" | "https" | _ => {
            Box::new(HttpSecretFetcher::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_secret_fetch_config_from_env() {
        unsafe {
            env::set_var("SECRETFS_URLS", "https://api.example.com/secrets,https://vault.example.com/v1/secret");
            env::set_var("SECRETFS_AUTH_TOKEN", "test-token-123");
            env::set_var("SECRETFS_TIMEOUT_SECONDS", "60");
        }
        
        let config = SecretFetchConfig::from_env().unwrap();
        
        assert_eq!(config.urls.len(), 2);
        assert_eq!(config.urls[0], "https://api.example.com/secrets");
        assert_eq!(config.auth_token, Some("test-token-123".to_string()));
        assert_eq!(config.timeout_seconds, 60);
        
        // Clean up
        unsafe {
            env::remove_var("SECRETFS_URLS");
            env::remove_var("SECRETFS_AUTH_TOKEN");
            env::remove_var("SECRETFS_TIMEOUT_SECONDS");
        }
    }
    
    #[test]
    fn test_mock_fetcher() {
        let fetcher = MockSecretFetcher::new();
        let config = SecretFetchConfig {
            urls: vec!["mock://test".to_string()],
            auth_token: None,
            timeout_seconds: 30,
            retry_attempts: 3,
            user_agent: "test".to_string(),
            headers: HashMap::new(),
        };
        
        let secrets = fetcher.fetch_secrets(&config).unwrap();
        assert_eq!(secrets.len(), 2);
        assert_eq!(secrets[0].key, "mock_api_key");
    }
    
    #[test]
    fn test_json_parsing() {
        let fetcher = HttpSecretFetcher::new();
        
        // Test flat object format
        let json = r#"{"api_key": "secret123", "db_password": "pass456"}"#;
        let secrets = fetcher.parse_json_secrets(json, "test://url").unwrap();
        assert_eq!(secrets.len(), 2);
        
        // Test array format
        let json = r#"[
            {"key": "api_key", "value": "secret123", "env": "prod"},
            {"key": "db_password", "value": "pass456", "env": "prod"}
        ]"#;
        let secrets = fetcher.parse_json_secrets(json, "test://url").unwrap();
        assert_eq!(secrets.len(), 2);
        assert_eq!(secrets[0].metadata.get("env"), Some(&"prod".to_string()));
    }
}
