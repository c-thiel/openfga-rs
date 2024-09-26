use http::{
    header::{ACCEPT, AUTHORIZATION},
    HeaderMap,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tonic::service::interceptor::Interceptor;

/// Refreshing openfga credentials failed.
#[derive(thiserror::Error, Debug)]
pub enum CredentialRefreshError {
    #[error("Could not build token fetch request: {0}")]
    InvalidConfiguration(#[source] reqwest::Error),
    #[error("Request to fetch token failed: {0}")]
    InvalidRequest(#[source] reqwest::Error),
    #[error("Non-retryable code {code} while fetching token. Body: {body}")]
    NonRetryableError { code: u16, body: String },
    #[error("Token Server error while refreshing token. Code {code}. Body: {body}")]
    ServerError { code: u16, body: String },
    #[error("Could not parse token response: {0}")]
    ParseError(#[source] reqwest::Error),
    #[error("Recieved token is not valid ASCII")]
    InvalidToken(String),
    #[error("Failed to start runtime for token refresh")]
    RuntimeError(#[source] tokio::io::Error),
    #[error("Could not join token fetch thread")]
    JoinError,
}

impl From<CredentialRefreshError> for tonic::Status {
    fn from(e: CredentialRefreshError) -> Self {
        let mut status = tonic::Status::internal("Could not refresh openfga credentials");
        status.set_source(Arc::new(e));
        status
    }
}

#[derive(veil::Redact, Clone)]
/// Client credentials used to authenticate with an `OAuth2` server [RFC 6749]
pub struct ClientCredentials {
    /// The client ID.
    pub client_id: String,
    /// The client secret.
    #[redact]
    pub client_secret: String,
    /// Endpoint used to perform the client credentials grant.
    /// Typically this is <issuer>/oauth2/token.
    pub token_endpoint: String,
    /// Extra headers to be added to each request.
    pub extra_headers: HeaderMap,
    /// Extra oauth parameters to be added to each authentication request.
    pub extra_oauth_params: HashMap<String, String>,
}

#[derive(Debug, Default, Clone)]
pub struct RefreshConfiguration {
    max_retry: u32,
    retry_interval: std::time::Duration,
}

#[derive(Debug)]
pub struct ClientCredentialInterceptor {
    inner: Arc<ClientCredentialIInterceptorInner>,
}

#[derive(Debug)]
struct ClientCredentialIInterceptorInner {
    credentials: ClientCredentials,
    refresh_config: RefreshConfiguration,
    state: RwLock<Option<ClientCredentialInterceptorState>>,
    client: reqwest::Client,
}

#[derive(veil::Redact)]
struct ClientCredentialInterceptorState {
    #[redact]
    token: String,
    token_expiry: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct TokenResponse {
    pub(super) access_token: String,
    pub(super) token_type: String,
    pub(super) expires_in: Option<u64>,
    pub(super) issued_token_type: Option<String>,
}

impl ClientCredentialInterceptor {
    /// Create a new [`ClientCredentialInterceptor`].
    /// The interceptor fetches a new token from the token endpoint
    /// and attaches it to intercepted requests. The token is
    /// refreshed automatically when it expires.
    #[must_use]
    pub fn new(credentials: ClientCredentials, refresh_config: RefreshConfiguration) -> Self {
        Self {
            inner: Arc::new(ClientCredentialIInterceptorInner {
                credentials,
                refresh_config,
                state: RwLock::new(None),
                client: reqwest::Client::new(),
            }),
        }
    }
}

/// Get a new token from the token endpoint
async fn get_token(
    credentials: &ClientCredentials,
    refresh_config: &RefreshConfiguration,
    client: &reqwest::Client,
) -> Result<TokenResponse, tonic::Status> {
    let ClientCredentials {
        client_id,
        client_secret,
        token_endpoint,
        extra_headers,
        extra_oauth_params,
    } = credentials;

    let RefreshConfiguration {
        max_retry,
        retry_interval,
    } = refresh_config;

    let mut params = HashMap::with_capacity(3 + extra_oauth_params.len());
    params.insert("grant_type", "client_credentials");
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.extend(
        extra_oauth_params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str())),
    );

    let mut counter = 0;
    let token = loop {
        counter += 1;

        let auth_request = client
            .request(http::Method::POST, token_endpoint)
            .header(ACCEPT, "application/json")
            .headers(extra_headers.clone())
            .form(&params)
            .build()
            .map_err(CredentialRefreshError::InvalidConfiguration)?;
        let auth_response = client
            .execute(auth_request)
            .await
            .map_err(CredentialRefreshError::InvalidRequest)?;

        match auth_response.status().as_u16() {
            500..=599 => {
                if counter > *max_retry {
                    return Err(CredentialRefreshError::ServerError {
                        code: auth_response.status().as_u16(),
                        body: auth_response.text().await.unwrap_or_default(),
                    }
                    .into());
                };

                // Retryable error
                tokio::time::sleep(*retry_interval).await;
            }
            200..=299 => {
                // Success
                let token_response: TokenResponse = auth_response
                    .json()
                    .await
                    .map_err(CredentialRefreshError::ParseError)?;

                break token_response;
            }
            _ => {
                // Non-retryable error
                return Err(CredentialRefreshError::NonRetryableError {
                    code: auth_response.status().as_u16(),
                    body: auth_response.text().await.unwrap_or_default(),
                }
                .into());
            }
        }
    };

    Ok(token)
}

impl Interceptor for ClientCredentialInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        let metadata = request.metadata_mut();
        if !metadata.contains_key(AUTHORIZATION.as_str()) {
            // Unwrap RWLock to propagate poison (writer panicked)
            let state_read_guard = self.inner.state.read().expect("poisoned lock");

            if let Some(ClientCredentialInterceptorState {
                token,
                token_expiry,
            }) = &*state_read_guard
            {
                if token_expiry > &chrono::Utc::now() {
                    metadata.insert(
                        AUTHORIZATION.as_str(),
                        format!("Bearer {token}")
                            .parse()
                            .map_err(|_e| CredentialRefreshError::InvalidToken(token.clone()))?,
                    );

                    return Ok(request);
                }
            };
            drop(state_read_guard);

            // Unwrap RWLock to propagate poison (writer panicked)
            // Get write lock immediately to not spawn multiple token fetch threads
            let mut state_write_guard = self.inner.state.write().unwrap();

            let credentials = self.inner.credentials.clone();
            let refresh_config = self.inner.refresh_config.clone();
            let client = self.inner.client.clone();

            let token_response = std::thread::spawn(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(CredentialRefreshError::RuntimeError)
                    .map(|rt| {
                        rt.block_on(async {
                            get_token(&credentials, &refresh_config, &client).await
                        })
                    })
            });

            let token_response = token_response
                .join()
                .map_err(|_e| CredentialRefreshError::JoinError)???;

            *state_write_guard = Some(ClientCredentialInterceptorState {
                token: token_response.access_token.clone(),
                // Default 59 minutes
                token_expiry: chrono::Utc::now()
                    + chrono::Duration::new(
                        i64::try_from(token_response.expires_in.unwrap_or(3600 - 60))
                            .unwrap_or(i64::MAX),
                        0,
                    )
                    .unwrap_or(chrono::Duration::try_seconds(3600 - 60).unwrap()),
            });
            drop(state_write_guard);

            metadata.insert(
                AUTHORIZATION.as_str(),
                format!("Bearer {}", token_response.access_token)
                    .parse()
                    .map_err(|_e| {
                        CredentialRefreshError::InvalidToken(token_response.access_token)
                    })?,
            );
        }

        Ok(request)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use http::header::CONTENT_TYPE;

    #[test]
    fn test_sync_context() {
        let mut oauth_server = mockito::Server::new();
        let url = oauth_server.url();
        let mock = oauth_server
            .mock("POST", mockito::Matcher::Any)
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex("grant_type=client_credentials".to_string()),
                mockito::Matcher::Regex("client_id=my-client".to_string()),
                mockito::Matcher::Regex("client_secret=my-secret".to_string()),
            ]))
            .with_status(200)
            .with_header(CONTENT_TYPE.as_str(), "application/json")
            .with_body(
                serde_json::json!({
                    "access_token": "my-issued-token",
                    "token_type": "my-token-type",
                    "expires_in": 3600
                })
                .to_string(),
            )
            .create();

        let mut interceptor = ClientCredentialInterceptor::new(
            ClientCredentials {
                client_id: "my-client".to_string(),
                client_secret: "my-secret".to_string(),
                token_endpoint: format!("{}/my-tenant/oauth2/token", url),
                extra_headers: HeaderMap::new(),
                extra_oauth_params: HashMap::new(),
            },
            RefreshConfiguration::default(),
        );

        let request = tonic::Request::new(());
        assert!(request.metadata().is_empty());
        let modified_request = interceptor.call(request).unwrap();

        let metadata = modified_request.metadata();
        assert!(metadata.contains_key("authorization"));
        assert_eq!(
            metadata.get("authorization").unwrap().to_str().unwrap(),
            "Bearer my-issued-token"
        );

        // verify mock was called
        mock.assert();
    }

    #[tokio::test]
    async fn test_async_context() {
        let mut oauth_server = mockito::Server::new_async().await;
        let url = oauth_server.url();
        let mock = oauth_server
            .mock("POST", mockito::Matcher::Any)
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex("grant_type=client_credentials".to_string()),
                mockito::Matcher::Regex("client_id=my-client".to_string()),
                mockito::Matcher::Regex("client_secret=my-secret".to_string()),
            ]))
            .with_status(200)
            .with_header(CONTENT_TYPE.as_str(), "application/json")
            .with_body(
                serde_json::json!({
                    "access_token": "my-issued-token",
                    "token_type": "my-token-type",
                    "expires_in": 3600
                })
                .to_string(),
            )
            .create();

        let mut interceptor = ClientCredentialInterceptor::new(
            ClientCredentials {
                client_id: "my-client".to_string(),
                client_secret: "my-secret".to_string(),
                token_endpoint: format!("{}/my-tenant/oauth2/token", url),
                extra_headers: HeaderMap::new(),
                extra_oauth_params: HashMap::new(),
            },
            RefreshConfiguration::default(),
        );

        let request = tonic::Request::new(());
        assert!(request.metadata().is_empty());
        let modified_request = interceptor.call(request).unwrap();

        let metadata = modified_request.metadata();
        assert!(metadata.contains_key("authorization"));
        assert_eq!(
            metadata.get("authorization").unwrap().to_str().unwrap(),
            "Bearer my-issued-token"
        );

        // verify mock was called
        mock.assert();
    }
}
