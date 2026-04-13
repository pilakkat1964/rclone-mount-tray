// Authentication handling for cloud services

use anyhow::{anyhow, Result};
use crate::models::{CloudService, AuthCredentials};
use std::collections::HashMap;

/// OAuth configuration for different services
pub struct OAuthConfig {
    pub service: CloudService,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

impl OAuthConfig {
    /// Get OAuth configuration for a service
    pub fn for_service(service: CloudService, client_id: String, client_secret: String) -> Result<Self> {
        let (auth_url, token_url, scopes) = match service {
            CloudService::GoogleDrive => (
                "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                "https://oauth2.googleapis.com/token".to_string(),
                vec![
                    "https://www.googleapis.com/auth/drive".to_string(),
                    "https://www.googleapis.com/auth/drive.appdata".to_string(),
                ],
            ),
            CloudService::OneDrive => (
                "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
                "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
                vec![
                    "offline_access".to_string(),
                    "Files.Read".to_string(),
                    "Files.ReadWrite".to_string(),
                ],
            ),
            _ => return Err(anyhow!("OAuth not supported for this service")),
        };

        Ok(Self {
            service,
            client_id,
            client_secret,
            auth_url,
            token_url,
            redirect_uri: "http://localhost:8888".to_string(),
            scopes,
        })
    }

    /// Generate authorization URL for user
    pub fn authorization_url(&self) -> String {
        let scope = self.scopes.join(" ");
        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline",
            self.auth_url, self.client_id, self.redirect_uri, urlencoding::encode(&scope)
        )
    }
}

/// Simple OAuth flow handler
pub struct OAuthHandler {
    config: OAuthConfig,
}

impl OAuthHandler {
    pub fn new(config: OAuthConfig) -> Self {
        Self { config }
    }

    /// Get authorization URL to display to user
    pub fn get_auth_url(&self) -> String {
        self.config.authorization_url()
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(&self, code: &str) -> Result<AuthCredentials> {
        let client = reqwest::Client::new();

        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", code);
        params.insert("client_id", &self.config.client_id);
        params.insert("client_secret", &self.config.client_secret);
        params.insert("redirect_uri", &self.config.redirect_uri);

        let response = client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await?;

        let token_response: TokenResponse = response.json().await?;

        let mut creds = AuthCredentials::new(self.config.service);
        creds.access_token = Some(token_response.access_token);
        creds.refresh_token = token_response.refresh_token;
        creds.token_expiry = token_response.expires_in.map(|exp| {
            let expiry_time = chrono::Local::now() + chrono::Duration::seconds(exp as i64);
            expiry_time.to_rfc3339()
        });

        Ok(creds)
    }

    /// Refresh expired access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<String> {
        let client = reqwest::Client::new();

        let mut params = HashMap::new();
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", refresh_token);
        params.insert("client_id", &self.config.client_id);
        params.insert("client_secret", &self.config.client_secret);

        let response = client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await?;

        let token_response: TokenResponse = response.json().await?;
        Ok(token_response.access_token)
    }
}

/// OAuth token response
#[derive(Debug, serde::Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: Option<String>,
}

/// Manual authentication handler (for direct token input)
pub struct ManualAuthHandler;

impl ManualAuthHandler {
    /// Create credentials from manually provided tokens
    pub fn create_credentials(
        service: CloudService,
        access_token: String,
    ) -> AuthCredentials {
        let mut creds = AuthCredentials::new(service);
        creds.access_token = Some(access_token);
        creds
    }

    /// Validate token format (basic check)
    pub fn validate_token(token: &str) -> Result<()> {
        if token.is_empty() {
            return Err(anyhow!("Token cannot be empty"));
        }

        if token.len() < 10 {
            return Err(anyhow!("Token appears too short"));
        }

        Ok(())
    }
}

/// Check if token needs refresh
pub fn is_token_expired(token_expiry: &Option<String>) -> bool {
    if let Some(expiry_str) = token_expiry {
        if let Ok(expiry_time) = chrono::DateTime::parse_from_rfc3339(expiry_str) {
            let now = chrono::Local::now();
            now > expiry_time.with_timezone(&chrono::Local)
        } else {
            false
        }
    } else {
        false
    }
}
