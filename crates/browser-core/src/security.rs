use anyhow::{Result, anyhow};
use keyring::Entry;
use validator::{Validate, ValidationError};
use ammonia::{Builder, UrlRelative};
use tracing::{info, warn, debug};
use argon2::{Argon2, PasswordHash, PasswordVerifier, PasswordHasher};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use regex::Regex;
use sha2::{Sha256, Digest};
use hex;

/// Security manager for handling credentials, validation, and sanitization
pub struct SecurityManager<'a> {
    /// HTML sanitizer for cleaning user input
    html_sanitizer: ammonia::Builder<'a>,
    /// URL validation regex
    url_regex: Regex,
    /// Password hasher
    argon2: Argon2<'static>,
}

impl<'a> SecurityManager<'a> {
    /// Create a new security manager with default configurations
    pub fn new() -> Self {
        // Configure HTML sanitizer
        let mut sanitizer = Builder::default();
        sanitizer
            .add_tags(&["a", "b", "i", "em", "strong", "u", "br", "p", "div", "span"])
            .add_generic_attributes(&["id", "class", "style"])
            .add_tag_attributes("a", &["href", "title", "target"])
            .add_tag_attributes("img", &["src", "alt", "width", "height"])
            .url_relative(UrlRelative::Deny)
            .link_rel(Some("noopener noreferrer"));

        // URL validation regex - more permissive to handle common URL formats
        let url_regex = Regex::new(r"^(https?|mailto):(//.+|.+)$").unwrap();

        Self {
            html_sanitizer: sanitizer,
            url_regex,
            argon2: Argon2::default(),
        }
    }

    /// Store a credential securely in the system keyring
    pub async fn store_credential(&self, service: &str, username: &str, password: &str) -> Result<()> {
        let entry = Entry::new(service, username)
            .map_err(|e| anyhow!("Failed to create keyring entry: {}", e))?;
        
        entry.set_password(password)
            .map_err(|e| anyhow!("Failed to store password in keyring: {}", e))?;
        
        info!("Stored credential for service: {}, user: {}", service, username);
        Ok(())
    }

    /// Retrieve a credential from the system keyring
    pub async fn get_credential(&self, service: &str, username: &str) -> Result<String> {
        let entry = Entry::new(service, username)
            .map_err(|e| anyhow!("Failed to create keyring entry: {}", e))?;
        
        let password = entry.get_password()
            .map_err(|e| {
                warn!("Failed to retrieve password from keyring: {}", e);
                anyhow!("Credential not found")
            })?;
        
        debug!("Retrieved credential for service: {}, user: {}", service, username);
        Ok(password)
    }

    /// Delete a credential from the system keyring
    pub async fn delete_credential(&self, service: &str, username: &str) -> Result<()> {
        let entry = Entry::new(service, username)
            .map_err(|e| anyhow!("Failed to create keyring entry: {}", e))?;
        
        entry.delete_credential()
            .map_err(|e| {
                warn!("Failed to delete password from keyring: {}", e);
                anyhow!("Failed to delete credential")
            })?;
        
        info!("Deleted credential for service: {}, user: {}", service, username);
        Ok(())
    }

    /// Hash a password using Argon2
    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?;
        
        Ok(password_hash.to_string())
    }

    /// Verify a password against its hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| anyhow!("Failed to parse password hash: {}", e))?;
        
        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Validate and sanitize a URL
    pub fn validate_url(&self, url: &str) -> Result<String> {
        // Basic regex validation
        if !self.url_regex.is_match(url) {
            return Err(anyhow!("Invalid URL format"));
        }

        // Additional validation using validator crate
        if !url::Url::parse(url).is_ok() {
            return Err(anyhow!("URL failed validation"));
        }

        Ok(url.to_string())
    }

    /// Sanitize HTML content to prevent XSS
    pub fn sanitize_html(&self, html: &str) -> String {
        self.html_sanitizer.clean(html).to_string()
    }

    /// Validate email address
    pub fn validate_email(&self, email: &str) -> Result<()> {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .map_err(|_| anyhow!("Invalid email regex"))?;
            
        if !email_regex.is_match(email) {
            return Err(anyhow!("Invalid email address"));
        }
        Ok(())
    }

    /// Validate IP address (IPv4 or IPv6)
    pub fn validate_ip(&self, ip: &str) -> Result<()> {
        if ip.parse::<std::net::IpAddr>().is_err() {
            return Err(anyhow!("Invalid IP address"));
        }
        Ok(())
    }

    /// Validate proxy configuration
    pub fn validate_proxy_config(&self, host: &str, port: u16, username: Option<&str>, password: Option<&str>) -> Result<()> {
        // Validate host (IP or domain)
        if host.parse::<std::net::IpAddr>().is_err() {
            // Try domain validation
            let domain_regex = Regex::new(r"^[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
                .map_err(|_| anyhow!("Invalid domain regex"))?;
                
            if !domain_regex.is_match(host) {
                return Err(anyhow!("Invalid proxy host: must be IP or domain"));
            }
        }

        // Validate port range
        if port == 0 || port > 65535 {
            return Err(anyhow!("Invalid proxy port: must be 1-65535"));
        }

        // Validate credentials if provided
        if let Some(user) = username {
            if user.is_empty() || user.len() > 255 {
                return Err(anyhow!("Invalid proxy username: must be 1-255 characters"));
            }
        }

        if let Some(pass) = password {
            if pass.len() > 1024 {
                return Err(anyhow!("Invalid proxy password: must be <= 1024 characters"));
            }
        }

        Ok(())
    }

    /// Generate a secure session token
    pub fn generate_session_token(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0), rand::random::<u64>()));
        hex::encode(hasher.finalize())
    }

    /// Check if a password meets security requirements
    pub fn validate_password_strength(&self, password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(anyhow!("Password must be at least 8 characters long"));
        }

        if password.len() > 128 {
            return Err(anyhow!("Password must be less than 128 characters"));
        }

        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

        if !has_upper {
            return Err(anyhow!("Password must contain at least one uppercase letter"));
        }

        if !has_lower {
            return Err(anyhow!("Password must contain at least one lowercase letter"));
        }

        if !has_digit {
            return Err(anyhow!("Password must contain at least one digit"));
        }

        if !has_special {
            return Err(anyhow!("Password must contain at least one special character"));
        }

        Ok(())
    }

    /// Create a content security policy header
    pub fn create_csp_header(&self, allow_custom_scripts: bool) -> String {
        let base_policy = "default-src 'self'; img-src 'self' data: https:; font-src 'self' data:; style-src 'self' 'unsafe-inline';";
        
        if allow_custom_scripts {
            format!("{} script-src 'self' 'unsafe-inline' 'unsafe-eval';", base_policy)
        } else {
            format!("{} script-src 'self';", base_policy)
        }
    }

    /// Sanitize user input for search queries
    pub fn sanitize_search_query(&self, query: &str) -> String {
        // Remove potential script injections and limit length
        let sanitized = self.sanitize_html(query);
        sanitized.chars().take(500).collect()
    }
}

impl Default for SecurityManager<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation structs for user input
#[derive(Debug, Validate)]
pub struct BookmarkInput {
    #[validate(length(min = 1, max = 255, message = "Title must be 1-255 characters"))]
    pub title: String,
    
    #[validate(url(message = "Invalid URL"))]
    pub url: String,
    
    #[validate(length(max = 100, message = "Folder must be <= 100 characters"))]
    pub folder: Option<String>,
}

#[derive(Debug, Validate)]
pub struct ProxyInput {
    #[validate(length(min = 1, max = 255, message = "Host must be 1-255 characters"))]
    pub host: String,
    
    #[validate(range(min = 1, max = 65535, message = "Port must be 1-65535"))]
    pub port: u16,
    
    #[validate(length(max = 255, message = "Username must be <= 255 characters"))]
    pub username: Option<String>,
    
    #[validate(length(max = 1024, message = "Password must be <= 1024 characters"))]
    pub password: Option<String>,
    
    #[validate(custom(function = "validate_proxy_protocol"))]
    pub protocol: String,
}

fn validate_proxy_protocol(protocol: &str) -> Result<(), ValidationError> {
    match protocol.to_lowercase().as_str() {
        "http" | "https" | "socks5" => Ok(()),
        _ => Err(ValidationError::new("Invalid proxy protocol")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_password_hashing() {
        let security = SecurityManager::new();
        let password = "TestPassword123!";
        
        let hash = security.hash_password(password).unwrap();
        assert!(security.verify_password(password, &hash).unwrap());
        assert!(!security.verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_url_validation() {
        let security = SecurityManager::new();
        
        assert!(security.validate_url("https://example.com").is_ok());
        assert!(security.validate_url("http://localhost:8080").is_ok());
        assert!(security.validate_url("mailto:test@example.com").is_ok());
        assert!(security.validate_url("javascript:alert('xss')").is_err());
        assert!(security.validate_url("not-a-url").is_err());
    }

    #[test]
    fn test_html_sanitization() {
        let security = SecurityManager::new();
        
        let safe_html = "<p>Hello <b>world</b>!</p>";
        let malicious_html = "<script>alert('xss')</script><p>Hello</p>";
        
        assert_eq!(security.sanitize_html(safe_html), safe_html);
        assert_eq!(security.sanitize_html(malicious_html), "<p>Hello</p>");
    }

    #[test]
    fn test_password_strength() {
        let security = SecurityManager::new();
        
        assert!(security.validate_password_strength("StrongP@ss123").is_ok());
        assert!(security.validate_password_strength("weak").is_err());
        assert!(security.validate_password_strength("nocaps123!").is_err());
        assert!(security.validate_password_strength("NOLOWER123!").is_err());
        assert!(security.validate_password_strength("NoNumbers!").is_err());
        assert!(security.validate_password_strength("NoSpecial123").is_err());
    }
}
