use super::{Address, Attachment, Message, MessageSummary, ProviderError, ProviderFailure};
use crate::core::ProviderId;
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://dropmail.me/api/graphql";

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DropMailCredentials {
    pub token: String,
    pub session_id: String,
    pub address_id: String,
    pub address: String,
    pub restore_key: String,
    pub expires_at: String,
}

impl std::fmt::Debug for DropMailCredentials {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DropMailCredentials")
            .field("token", &"[REDACTED]")
            .field("session_id", &self.session_id)
            .field("address_id", &self.address_id)
            .field("address", &self.address)
            .field("restore_key", &"[REDACTED]")
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

#[derive(Clone)]
pub struct DropMail {
    client: Client,
    endpoint: String,
    origin: String,
}

impl std::fmt::Debug for DropMail {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DropMail")
            .field("endpoint", &"[REDACTED]")
            .field("origin", &self.origin)
            .finish_non_exhaustive()
    }
}

impl DropMail {
    pub fn new(token: &str, timeout: Duration) -> Result<Self, ProviderError> {
        Self::with_base_url(DEFAULT_BASE_URL, token, timeout)
    }

    pub fn with_base_url(
        base_url: impl Into<String>,
        token: &str,
        timeout: Duration,
    ) -> Result<Self, ProviderError> {
        if !token.starts_with("af_") || token.len() < 12 {
            return Err(error(
                ProviderFailure::Authentication,
                "DropMail requires a current af_ token",
            ));
        }
        let base = base_url.into().trim_end_matches('/').to_owned();
        let parsed = reqwest::Url::parse(&base).map_err(|_| {
            error(
                ProviderFailure::InvalidResponse,
                "invalid DropMail endpoint",
            )
        })?;
        let host = parsed.host_str().ok_or_else(|| {
            error(
                ProviderFailure::InvalidResponse,
                "invalid DropMail endpoint",
            )
        })?;
        let origin = match parsed.port() {
            Some(port) => format!("{}://{host}:{port}", parsed.scheme()),
            None => format!("{}://{host}", parsed.scheme()),
        };
        let client = Client::builder()
            .timeout(timeout)
            .user_agent(concat!("tmpmail/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| {
                error(
                    ProviderFailure::Unavailable,
                    "could not initialize HTTP client",
                )
            })?;
        Ok(Self {
            client,
            endpoint: format!("{base}/{token}"),
            origin,
        })
    }

    pub async fn create(&self) -> Result<DropMailCredentials, ProviderError> {
        let data: IntroduceData = self
            .graphql(
                "mutation CreateSession { introduceSession { id expiresAt addresses { id address restoreKey } } }",
                json!({}),
            )
            .await?;
        let address = data
            .introduce_session
            .addresses
            .into_iter()
            .next()
            .ok_or_else(|| {
                error(
                    ProviderFailure::InvalidResponse,
                    "DropMail returned no address",
                )
            })?;
        Ok(DropMailCredentials {
            token: self.token_from_endpoint(),
            session_id: data.introduce_session.id,
            address_id: address.id,
            address: address.address,
            restore_key: address.restore_key,
            expires_at: data.introduce_session.expires_at,
        })
    }

    pub async fn domains(&self) -> Result<Vec<String>, ProviderError> {
        let data: DomainsData = self
            .graphql(
                "query Domains { domains { name availableVia expiresAt health } }",
                json!({}),
            )
            .await?;
        Ok(data
            .domains
            .into_iter()
            .filter(|domain| {
                domain.health != "BROKEN" && domain.available_via.iter().any(|value| value == "API")
            })
            .map(|domain| domain.name)
            .collect())
    }

    pub async fn messages(
        &self,
        credentials: &mut DropMailCredentials,
    ) -> Result<Vec<MessageSummary>, ProviderError> {
        let session = self.session(credentials).await?;
        credentials.expires_at = session.expires_at;
        Ok(session.mails.iter().map(summary).collect())
    }

    pub async fn message(
        &self,
        credentials: &mut DropMailCredentials,
        id: &str,
    ) -> Result<Message, ProviderError> {
        let session = self.session(credentials).await?;
        credentials.expires_at = session.expires_at;
        session
            .mails
            .into_iter()
            .find(|mail| mail.id == id)
            .map(full_message)
            .ok_or_else(|| error(ProviderFailure::NotFound, "DropMail message not found"))
    }

    pub async fn delete_address(
        &self,
        credentials: &DropMailCredentials,
    ) -> Result<(), ProviderError> {
        let data: DeleteData = self
            .graphql(
                "mutation DeleteAddress($id: ID!) { deleteAddress(input: {addressId: $id}) }",
                json!({"id": credentials.address_id}),
            )
            .await?;
        if data.delete_address {
            Ok(())
        } else {
            Err(error(
                ProviderFailure::InvalidResponse,
                "DropMail did not remove the address",
            ))
        }
    }

    pub async fn restore_address(
        &self,
        session_id: &str,
        address: &str,
        restore_key: &str,
    ) -> Result<(), ProviderError> {
        let _: RestoreData = self
            .graphql(
                "mutation RestoreAddress($session: ID!, $address: String!, $key: String!) { restoreAddress(input: {sessionId: $session, mailAddress: $address, restoreKey: $key}) { id } }",
                json!({"session": session_id, "address": address, "key": restore_key}),
            )
            .await?;
        Ok(())
    }

    pub async fn attachment(&self, download_url: &str) -> Result<Response, ProviderError> {
        let trusted = reqwest::Url::parse(download_url)
            .ok()
            .and_then(|url| {
                let host = url.host_str()?;
                Some(match url.port() {
                    Some(port) => format!("{}://{host}:{port}", url.scheme()),
                    None => format!("{}://{host}", url.scheme()),
                })
            })
            .is_some_and(|origin| origin == self.origin);
        if !trusted {
            return Err(error(
                ProviderFailure::InvalidResponse,
                "DropMail returned an unsafe attachment URL",
            ));
        }
        self.response(self.client.get(download_url).send().await)
            .await
    }

    async fn session(
        &self,
        credentials: &DropMailCredentials,
    ) -> Result<SessionResource, ProviderError> {
        let data: SessionData = self
            .graphql(
                "query Session($id: ID!) { session(id: $id) { id expiresAt mails { id rawSize fromAddr toAddr receivedAt text html headerSubject downloadUrl attachments { id name mime rawSize downloadUrl } } } }",
                json!({"id": credentials.session_id}),
            )
            .await?;
        data.session.ok_or_else(|| {
            error(
                ProviderFailure::NotFound,
                "DropMail session expired or was not found",
            )
        })
    }

    async fn graphql<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T, ProviderError> {
        let response = self
            .response(
                self.client
                    .post(&self.endpoint)
                    .json(&json!({"query": query, "variables": variables}))
                    .send()
                    .await,
            )
            .await?;
        let is_json = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("application/json"));
        if !is_json {
            return Err(error(
                ProviderFailure::InvalidResponse,
                "DropMail returned an unexpected content type",
            ));
        }
        let envelope: GraphQlEnvelope<T> = response.json().await.map_err(|_| {
            error(
                ProviderFailure::InvalidResponse,
                "DropMail returned malformed JSON",
            )
        })?;
        if let Some(first) = envelope.errors.first() {
            return Err(graphql_error(first));
        }
        envelope.data.ok_or_else(|| {
            error(
                ProviderFailure::InvalidResponse,
                "DropMail response omitted data",
            )
        })
    }

    async fn response(
        &self,
        response: Result<Response, reqwest::Error>,
    ) -> Result<Response, ProviderError> {
        let response = response.map_err(|failure| {
            if failure.is_timeout() {
                error(ProviderFailure::Timeout, "DropMail request timed out")
            } else {
                error(ProviderFailure::Unavailable, "DropMail is unavailable")
            }
        })?;
        match response.status() {
            status if status.is_success() => Ok(response),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(error(
                ProviderFailure::Authentication,
                "DropMail token is invalid or expired",
            )),
            StatusCode::TOO_MANY_REQUESTS => Err(error(
                ProviderFailure::RateLimited {
                    retry_after_seconds: response
                        .headers()
                        .get(reqwest::header::RETRY_AFTER)
                        .and_then(|value| value.to_str().ok())
                        .and_then(|value| value.parse().ok()),
                },
                "DropMail rate limit reached",
            )),
            status if status.is_server_error() => Err(error(
                ProviderFailure::Unavailable,
                "DropMail is unavailable",
            )),
            _ => Err(error(
                ProviderFailure::InvalidResponse,
                "DropMail returned an unexpected response",
            )),
        }
    }

    fn token_from_endpoint(&self) -> String {
        self.endpoint
            .rsplit('/')
            .next()
            .unwrap_or_default()
            .to_owned()
    }
}

fn graphql_error(error_value: &GraphQlError) -> ProviderError {
    let code = error_value
        .extensions
        .as_ref()
        .and_then(|value| value.code.as_deref())
        .unwrap_or_default();
    match code {
        "SESSION_NOT_FOUND" => error(
            ProviderFailure::NotFound,
            "DropMail session expired or was not found",
        ),
        "RATE_LIMIT_EXCEEDED" => error(
            ProviderFailure::RateLimited {
                retry_after_seconds: None,
            },
            "DropMail rate limit reached; automated CAPTCHA solving is disabled",
        ),
        "authentication_error" | "token_expired" | "legacy_token_disabled" => error(
            ProviderFailure::Authentication,
            "DropMail token is invalid or expired",
        ),
        _ => error(
            ProviderFailure::InvalidResponse,
            "DropMail GraphQL operation failed",
        ),
    }
}

fn error(failure: ProviderFailure, message: impl Into<String>) -> ProviderError {
    ProviderError::for_provider(ProviderId::DropMail, failure, message)
}

fn summary(mail: &MailResource) -> MessageSummary {
    MessageSummary {
        id: mail.id.clone(),
        from: Address {
            name: String::new(),
            address: mail.from_addr.clone(),
        },
        subject: mail.header_subject.clone().unwrap_or_default(),
        intro: mail
            .text
            .as_deref()
            .unwrap_or_default()
            .chars()
            .take(120)
            .collect(),
        seen: false,
        has_attachments: !mail.attachments.is_empty(),
        size: mail.raw_size,
        created_at: mail.received_at.clone(),
    }
}

fn full_message(mail: MailResource) -> Message {
    Message {
        id: mail.id,
        from: Address {
            name: String::new(),
            address: mail.from_addr,
        },
        subject: mail.header_subject.unwrap_or_default(),
        text: mail.text.unwrap_or_default(),
        html: mail.html.into_iter().collect(),
        seen: false,
        attachments: mail.attachments.into_iter().map(Into::into).collect(),
        created_at: mail.received_at,
    }
}

#[derive(Deserialize)]
struct GraphQlEnvelope<T> {
    data: Option<T>,
    #[serde(default)]
    errors: Vec<GraphQlError>,
}

#[derive(Deserialize)]
struct GraphQlError {
    #[allow(dead_code)]
    message: String,
    extensions: Option<GraphQlExtensions>,
}

#[derive(Deserialize)]
struct GraphQlExtensions {
    code: Option<String>,
}

#[derive(Deserialize)]
struct IntroduceData {
    #[serde(rename = "introduceSession")]
    introduce_session: SessionCreation,
}

#[derive(Deserialize)]
struct SessionCreation {
    id: String,
    #[serde(rename = "expiresAt")]
    expires_at: String,
    addresses: Vec<AddressResource>,
}

#[derive(Deserialize)]
struct AddressResource {
    id: String,
    address: String,
    #[serde(rename = "restoreKey")]
    restore_key: String,
}

#[derive(Deserialize)]
struct SessionData {
    session: Option<SessionResource>,
}

#[derive(Deserialize)]
struct SessionResource {
    #[allow(dead_code)]
    id: String,
    #[serde(rename = "expiresAt")]
    expires_at: String,
    #[serde(default)]
    mails: Vec<MailResource>,
}

#[derive(Deserialize)]
struct MailResource {
    id: String,
    #[serde(rename = "rawSize")]
    raw_size: u64,
    #[serde(rename = "fromAddr")]
    from_addr: String,
    #[allow(dead_code)]
    #[serde(rename = "toAddr")]
    to_addr: String,
    #[serde(rename = "receivedAt")]
    received_at: String,
    text: Option<String>,
    html: Option<String>,
    #[serde(default, rename = "headerSubject")]
    header_subject: Option<String>,
    #[allow(dead_code)]
    #[serde(default, rename = "downloadUrl")]
    download_url: String,
    #[serde(default)]
    attachments: Vec<AttachmentResource>,
}

#[derive(Deserialize)]
struct AttachmentResource {
    id: String,
    name: Option<String>,
    mime: String,
    #[serde(rename = "rawSize")]
    raw_size: u64,
    #[serde(rename = "downloadUrl")]
    download_url: String,
}

impl From<AttachmentResource> for Attachment {
    fn from(value: AttachmentResource) -> Self {
        Self {
            id: value.id,
            filename: value.name.unwrap_or_else(|| "attachment".to_owned()),
            content_type: value.mime,
            size: value.raw_size,
            download_url: value.download_url,
        }
    }
}

#[derive(Deserialize)]
struct DeleteData {
    #[serde(rename = "deleteAddress")]
    delete_address: bool,
}

#[derive(Deserialize)]
struct RestoreData {
    #[allow(dead_code)]
    #[serde(rename = "restoreAddress")]
    restore_address: RestoreResource,
}

#[derive(Deserialize)]
struct RestoreResource {
    #[allow(dead_code)]
    id: String,
}

#[derive(Deserialize)]
struct DomainsData {
    domains: Vec<DomainResource>,
}

#[derive(Deserialize)]
struct DomainResource {
    name: String,
    #[serde(rename = "availableVia")]
    available_via: Vec<String>,
    #[allow(dead_code)]
    #[serde(rename = "expiresAt")]
    expires_at: Option<String>,
    health: String,
}
