use crate::AccessTokenExpired;
use std::fmt::{Debug, Display};

/// A trait for an auth provider that can provide
/// a client id
pub trait ClientIdProvider {
    /// Get the client id for this auth provider
    fn client_id(&self) -> &str;
}

/// A simple client id provider that simply wraps the client id string
#[derive(Debug, Clone)]
pub struct ClientId(pub String);

impl ClientId {
    /// Create a new ClientId wrapper with the given string
    pub fn new(client_id: impl Into<String>) -> Self {
        Self(client_id.into())
    }
}

impl ClientIdProvider for ClientId {
    fn client_id(&self) -> &str {
        &self.0
    }
}

/// Represents an access token
#[derive(Debug, Clone)]
pub enum AccessToken {
    /// Access token
    Token(String),

    /// Access token expired or otherwise needs refreshing
    NeedsRefresh,
}

impl From<String> for AccessToken {
    fn from(token: String) -> Self {
        Self::Token(token)
    }
}

/// A trait for an auth provider that can provide
/// an access token.
#[async_trait::async_trait]
pub trait AccessTokenProvider: ClientIdProvider {
    /// Error type used for refreshing errors
    type Error: Display + Debug;

    /// Get the access token for this auth provider.
    ///
    /// If the token is expired, this should return AccessToken::NeedsRefresh to indicate
    /// that [`AccessTokenProvider::refresh_token`] should be called
    fn access_token(&self) -> AccessToken;

    /// Refresh the token.
    async fn refresh_token(&self) -> Result<String, Self::Error>;
}

/// A simple access token provider that errors if refreshing is attempted. It is strongly advised
/// that you implement your own [`AccessTokenProvider`] so that you can handle refreshing.
#[derive(Debug)]
pub struct AccessTokenOnly {
    client_id: String,
    token: String,
}

impl AccessTokenOnly {
    /// Creat a new [`AccessTokenOnly`] instance with the given client id and access token
    pub fn new(client_id: impl Into<String>, access_token: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            token: access_token.into(),
        }
    }
}

impl ClientIdProvider for AccessTokenOnly {
    fn client_id(&self) -> &str {
        &self.client_id
    }
}

#[async_trait::async_trait]
impl AccessTokenProvider for AccessTokenOnly {
    type Error = AccessTokenExpired;

    fn access_token(&self) -> AccessToken {
        AccessToken::Token(self.token.clone())
    }

    async fn refresh_token(&self) -> Result<String, Self::Error> {
        Err(AccessTokenExpired)
    }
}

/// Obtain an access token from an AccessTokenProvider
#[macro_export]
#[doc(hidden)]
macro_rules! access_token {
    ($auth: expr, $error_type: ident) => {
        match $auth.access_token() {
            crate::auth::AccessToken::Token(token) => token,
            crate::auth::AccessToken::NeedsRefresh => $auth
                .refresh_token()
                .await
                .map_err($error_type::RefreshToken)?,
        }
    };
}
