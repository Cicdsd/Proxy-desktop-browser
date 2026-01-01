use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::async_runtime::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    User,
    Admin,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: chrono::DateTime<Utc>,
    pub last_login: Option<chrono::DateTime<Utc>>,
    pub enterprise_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub username: String,
    pub role: UserRole,
    pub enterprise_id: Option<String>,
    pub exp: i64, // Expiration time
    pub iat: i64, // Issued at
    pub jti: String, // JWT ID
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshToken {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub created_at: chrono::DateTime<Utc>,
}

pub struct AuthManager {
    jwt_secret: String,
    users: Arc<RwLock<HashMap<String, User>>>, // In-memory for demo, use DB in production
    password_hashes: Arc<RwLock<HashMap<String, String>>>, // user_id -> password_hash
    refresh_tokens: Arc<RwLock<HashMap<String, RefreshToken>>>,
    argon2: Argon2<'static>,

}

impl AuthManager {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret,
            users: Arc::new(RwLock::new(HashMap::new())),
            password_hashes: Arc::new(RwLock::new(HashMap::new())),
            refresh_tokens: Arc::new(RwLock::new(HashMap::new())),
            argon2: Argon2::default(),
        }
    }


    /// Register a new user
    pub async fn register(&self, username: String, email: String, password: String) -> Result<User> {
        // Validate input
        if username.len() < 3 {
            return Err(anyhow!("Username must be at least 3 characters"));
        }
        if !email.contains('@') {
            return Err(anyhow!("Invalid email format"));
        }
        if password.len() < 8 {
            return Err(anyhow!("Password must be at least 8 characters"));
        }

        // Check if user exists
        let users = self.users.read().await;
        if users.values().any(|u| u.username == username || u.email == email) {
            return Err(anyhow!("User already exists"));
        }
        drop(users);

        // Hash password
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        // Create user (in production, store in database)
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: username.clone(),
            email,
            role: UserRole::User,
            created_at: Utc::now(),
            last_login: None,
            enterprise_id: None,
        };

        // Store user and password hash
        let mut users = self.users.write().await;
        users.insert(user.id.clone(), user.clone());
        drop(users);

        // Store password hash separately for security
        let mut hashes = self.password_hashes.write().await;
        hashes.insert(user.id.clone(), password_hash);

        Ok(user)
    }


    /// Authenticate user and return tokens
    pub async fn login(&self, username: String, password: String) -> Result<(String, String)> {
        // Find user
        let users = self.users.read().await;
        let user = users
            .values()
            .find(|u| u.username == username)
            .ok_or_else(|| anyhow!("Invalid credentials"))?
            .clone();
        drop(users);

        // Verify password hash
        let hashes = self.password_hashes.read().await;
        let stored_hash = hashes
            .get(&user.id)
            .ok_or_else(|| anyhow!("Invalid credentials"))?;
        
        let parsed_hash = PasswordHash::new(stored_hash)
            .map_err(|_| anyhow!("Invalid credentials"))?;
        
        self.argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| anyhow!("Invalid credentials"))?;
        drop(hashes);

        // Update last login

        let mut users = self.users.write().await;
        if let Some(stored_user) = users.get_mut(&user.id) {
            stored_user.last_login = Some(Utc::now());
        }

        // Generate tokens
        let access_token = self.generate_access_token(&user)?;
        let refresh_token = self.generate_refresh_token(&user.id).await?;

        Ok((access_token, refresh_token))
    }

    /// Refresh access token
    pub async fn refresh_access_token(&self, refresh_token: String) -> Result<String> {
        // Verify refresh token
        let tokens = self.refresh_tokens.read().await;
        let token_data = tokens
            .values()
            .find(|t| self.verify_refresh_token(&refresh_token, t))
            .ok_or_else(|| anyhow!("Invalid refresh token"))?
            .clone();
        drop(tokens);

        // Check if expired
        if token_data.expires_at < Utc::now() {
            return Err(anyhow!("Refresh token expired"));
        }

        // Get user
        let users = self.users.read().await;
        let user = users
            .get(&token_data.user_id)
            .ok_or_else(|| anyhow!("User not found"))?
            .clone();

        // Generate new access token
        self.generate_access_token(&user)
    }

    /// Validate JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    /// Generate access token
    fn generate_access_token(&self, user: &User) -> Result<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            enterprise_id: user.enterprise_id.clone(),
            exp: (now + Duration::minutes(15)).timestamp(), // 15 minutes
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        Ok(token)
    }

    /// Generate refresh token
    async fn generate_refresh_token(&self, user_id: &str) -> Result<String> {
        let token_id = Uuid::new_v4().to_string();
        let token_string = Uuid::new_v4().to_string();
        
        // Hash the refresh token
        let salt = SaltString::generate(&mut OsRng);
        let token_hash = self
            .argon2
            .hash_password(token_string.as_bytes(), &salt)?
            .to_string();

        let refresh_token = RefreshToken {
            id: token_id,
            user_id: user_id.to_string(),
            token_hash,
            expires_at: Utc::now() + Duration::days(30), // 30 days
            created_at: Utc::now(),
        };

        // Store refresh token
        let mut tokens = self.refresh_tokens.write().await;
        tokens.insert(token_string.clone(), refresh_token);

        Ok(token_string)
    }

    /// Verify refresh token
    fn verify_refresh_token(&self, token: &str, stored: &RefreshToken) -> bool {
        let parsed_hash = PasswordHash::new(&stored.token_hash);
        match parsed_hash {
            Ok(hash) => self.argon2.verify_password(token.as_bytes(), &hash).is_ok(),
            Err(_) => false,
        }
    }

    /// Revoke refresh token
    pub async fn revoke_refresh_token(&self, token: String) -> Result<()> {
        let mut tokens = self.refresh_tokens.write().await;
        tokens.remove(&token);
        Ok(())
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Option<User> {
        let users = self.users.read().await;
        users.get(user_id).cloned()
    }

    /// Create enterprise user
    pub async fn create_enterprise_user(
        &self,
        username: String,
        email: String,
        password: String,
        enterprise_id: String,
    ) -> Result<User> {
        // Similar to register but with enterprise role
        if username.len() < 3 {
            return Err(anyhow!("Username must be at least 3 characters"));
        }
        if !email.contains('@') {
            return Err(anyhow!("Invalid email format"));
        }
        if password.len() < 8 {
            return Err(anyhow!("Password must be at least 8 characters"));
        }

        let users = self.users.read().await;
        if users.values().any(|u| u.username == username || u.email == email) {
            return Err(anyhow!("User already exists"));
        }
        drop(users);

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        let user = User {
            id: Uuid::new_v4().to_string(),
            username: username.clone(),
            email,
            role: UserRole::Enterprise,
            created_at: Utc::now(),
            last_login: None,
            enterprise_id: Some(enterprise_id),
        };

        // Store user and password hash
        let mut users = self.users.write().await;
        users.insert(user.id.clone(), user.clone());
        drop(users);

        // Store password hash
        let mut hashes = self.password_hashes.write().await;
        hashes.insert(user.id.clone(), password_hash);

        Ok(user)
    }


    /// Promote user to admin
    pub async fn promote_to_admin(&self, user_id: &str) -> Result<()> {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(user_id) {
            user.role = UserRole::Admin;
            Ok(())
        } else {
            Err(anyhow!("User not found"))
        }
    }
}

// Tauri command handlers
#[tauri::command]
pub async fn register_user(
    auth: tauri::State<'_, Arc<AuthManager>>,
    username: String,
    email: String,
    password: String,
) -> Result<User, String> {
    auth.register(username, email, password)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn login_user(
    auth: tauri::State<'_, Arc<AuthManager>>,
    username: String,
    password: String,
) -> Result<(String, String), String> {
    auth.login(username, password)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn refresh_token(
    auth: tauri::State<'_, Arc<AuthManager>>,
    refresh_token: String,
) -> Result<String, String> {
    auth.refresh_access_token(refresh_token)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn validate_jwt_token(
    auth: tauri::State<'_, Arc<AuthManager>>,
    token: String,
) -> Result<Claims, String> {
    auth.validate_token(&token)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn logout(
    auth: tauri::State<'_, Arc<AuthManager>>,
    refresh_token: String,
) -> Result<(), String> {
    auth.revoke_refresh_token(refresh_token)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_current_user(
    auth: tauri::State<'_, Arc<AuthManager>>,
    token: String,
) -> Result<Option<User>, String> {
    let claims = auth.validate_token(&token).map_err(|e| e.to_string())?;
    Ok(auth.get_user(&claims.sub).await)
}

#[tauri::command]
pub async fn create_enterprise_user(
    auth: tauri::State<'_, Arc<AuthManager>>,
    username: String,
    email: String,
    password: String,
    enterprise_id: String,
) -> Result<User, String> {
    auth.create_enterprise_user(username, email, password, enterprise_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn promote_user_to_admin(
    auth: tauri::State<'_, Arc<AuthManager>>,
    user_id: String,
) -> Result<(), String> {
    auth.promote_to_admin(&user_id)
        .await
        .map_err(|e| e.to_string())
}
