use super::{Address, Attachment, Message, MessageSummary, ProviderError, ProviderFailure};
use rand::distr::{Alphanumeric, SampleString};
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.mail.tm";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailTmCredentials {
    pub account_id: String,
    pub address: String,
    pub password: String,
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct MailTm {
    client: Client,
    base_url: String,
}

impl MailTm {
    pub fn new(timeout: Duration) -> Result<Self, ProviderError> {
        Self::with_base_url(DEFAULT_BASE_URL, timeout)
    }

    pub fn with_base_url(
        base_url: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(timeout)
            .user_agent(concat!("tmpmail/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| {
                ProviderError::new(
                    ProviderFailure::Unavailable,
                    "could not initialize HTTP client",
                )
            })?;
        Ok(Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_owned(),
        })
    }

    pub async fn domains(&self) -> Result<Vec<String>, ProviderError> {
        let response: Collection<DomainResource> = self
            .json(self.client.get(self.url("/domains")).send().await)
            .await?;
        Ok(response
            .members
            .into_iter()
            .filter(|domain| domain.is_active)
            .map(|domain| domain.domain)
            .collect())
    }

    pub async fn create(
        &self,
        address: &str,
        password: &str,
    ) -> Result<MailTmCredentials, ProviderError> {
        let account: AccountResource = self
            .json(
                self.client
                    .post(self.url("/accounts"))
                    .json(&CredentialsRequest { address, password })
                    .send()
                    .await,
            )
            .await?;
        let token = self.authenticate(address, password).await?;
        Ok(MailTmCredentials {
            account_id: account.id,
            address: account.address,
            password: password.to_owned(),
            token,
        })
    }

    pub async fn create_random(
        &self,
        local_part: Option<&str>,
    ) -> Result<MailTmCredentials, ProviderError> {
        let domain = self.domains().await?.into_iter().next().ok_or_else(|| {
            ProviderError::new(
                ProviderFailure::Unavailable,
                "Mail.tm has no active domains",
            )
        })?;
        let local = local_part
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| random_string(12).to_ascii_lowercase());
        let password = random_string(32);
        self.create(&format!("{local}@{domain}"), &password).await
    }

    pub async fn authenticate(
        &self,
        address: &str,
        password: &str,
    ) -> Result<String, ProviderError> {
        let response: TokenResponse = self
            .json(
                self.client
                    .post(self.url("/token"))
                    .json(&CredentialsRequest { address, password })
                    .send()
                    .await,
            )
            .await?;
        Ok(response.token)
    }

    pub async fn refresh(&self, credentials: &mut MailTmCredentials) -> Result<(), ProviderError> {
        credentials.token = self
            .authenticate(&credentials.address, &credentials.password)
            .await?;
        Ok(())
    }

    pub async fn messages(&self, token: &str) -> Result<Vec<MessageSummary>, ProviderError> {
        let response: Collection<MessageListResource> = self
            .json(
                self.client
                    .get(self.url("/messages"))
                    .bearer_auth(token)
                    .send()
                    .await,
            )
            .await?;
        Ok(response.members.into_iter().map(Into::into).collect())
    }

    pub async fn messages_with_refresh(
        &self,
        credentials: &mut MailTmCredentials,
    ) -> Result<Vec<MessageSummary>, ProviderError> {
        match self.messages(&credentials.token).await {
            Err(error) if error.failure == ProviderFailure::Authentication => {
                self.refresh(credentials).await?;
                self.messages(&credentials.token).await
            }
            result => result,
        }
    }

    pub async fn message(&self, token: &str, id: &str) -> Result<Message, ProviderError> {
        let response: MessageResource = self
            .json(
                self.client
                    .get(self.url(&format!("/messages/{id}")))
                    .bearer_auth(token)
                    .send()
                    .await,
            )
            .await?;
        Ok(response.into())
    }

    pub async fn message_with_refresh(
        &self,
        credentials: &mut MailTmCredentials,
        id: &str,
    ) -> Result<Message, ProviderError> {
        match self.message(&credentials.token, id).await {
            Err(error) if error.failure == ProviderFailure::Authentication => {
                self.refresh(credentials).await?;
                self.message(&credentials.token, id).await
            }
            result => result,
        }
    }

    pub async fn acknowledge(&self, token: &str, id: &str) -> Result<(), ProviderError> {
        self.empty(
            self.client
                .patch(self.url(&format!("/messages/{id}")))
                .bearer_auth(token)
                .json(&serde_json::json!({"seen": true}))
                .send()
                .await,
        )
        .await
    }

    pub async fn acknowledge_with_refresh(
        &self,
        credentials: &mut MailTmCredentials,
        id: &str,
    ) -> Result<(), ProviderError> {
        match self.acknowledge(&credentials.token, id).await {
            Err(error) if error.failure == ProviderFailure::Authentication => {
                self.refresh(credentials).await?;
                self.acknowledge(&credentials.token, id).await
            }
            result => result,
        }
    }

    pub async fn delete_account(&self, token: &str, id: &str) -> Result<(), ProviderError> {
        self.empty(
            self.client
                .delete(self.url(&format!("/accounts/{id}")))
                .bearer_auth(token)
                .send()
                .await,
        )
        .await
    }

    pub async fn attachment(
        &self,
        token: &str,
        download_url: &str,
    ) -> Result<Response, ProviderError> {
        let url = if download_url.starts_with('/') {
            self.url(download_url)
        } else if download_url.starts_with(&self.base_url) {
            download_url.to_owned()
        } else {
            return Err(ProviderError::new(
                ProviderFailure::InvalidResponse,
                "Mail.tm returned an unsafe attachment URL",
            ));
        };
        self.response(self.client.get(url).bearer_auth(token).send().await)
            .await
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    async fn json<T: DeserializeOwned>(
        &self,
        response: Result<Response, reqwest::Error>,
    ) -> Result<T, ProviderError> {
        let response = self.response(response).await?;
        let is_json = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| {
                value.starts_with("application/json") || value.starts_with("application/ld+json")
            });
        if !is_json {
            return Err(ProviderError::new(
                ProviderFailure::InvalidResponse,
                "Mail.tm returned an unexpected content type",
            ));
        }
        response.json().await.map_err(|_| {
            ProviderError::new(
                ProviderFailure::InvalidResponse,
                "Mail.tm returned malformed JSON",
            )
        })
    }

    async fn empty(&self, response: Result<Response, reqwest::Error>) -> Result<(), ProviderError> {
        self.response(response).await.map(|_| ())
    }

    async fn response(
        &self,
        response: Result<Response, reqwest::Error>,
    ) -> Result<Response, ProviderError> {
        let response = response.map_err(|error| {
            if error.is_timeout() {
                ProviderError::new(ProviderFailure::Timeout, "Mail.tm request timed out")
            } else {
                ProviderError::new(ProviderFailure::Unavailable, "Mail.tm is unavailable")
            }
        })?;
        match response.status() {
            status if status.is_success() => Ok(response),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(ProviderError::new(
                ProviderFailure::Authentication,
                "Mail.tm authentication failed",
            )),
            StatusCode::NOT_FOUND => Err(ProviderError::new(
                ProviderFailure::NotFound,
                "Mail.tm resource not found",
            )),
            StatusCode::CONFLICT | StatusCode::UNPROCESSABLE_ENTITY => Err(ProviderError::new(
                ProviderFailure::Conflict,
                "Mail.tm rejected the requested address",
            )),
            StatusCode::TOO_MANY_REQUESTS => Err(ProviderError::new(
                ProviderFailure::RateLimited {
                    retry_after_seconds: response
                        .headers()
                        .get(reqwest::header::RETRY_AFTER)
                        .and_then(|value| value.to_str().ok())
                        .and_then(|value| value.parse().ok()),
                },
                "Mail.tm rate limit reached",
            )),
            status if status.is_server_error() => Err(ProviderError::new(
                ProviderFailure::Unavailable,
                "Mail.tm is unavailable",
            )),
            _ => Err(ProviderError::new(
                ProviderFailure::InvalidResponse,
                "Mail.tm returned an unexpected response",
            )),
        }
    }
}

fn random_string(length: usize) -> String {
    Alphanumeric.sample_string(&mut rand::rng(), length)
}

#[derive(Serialize)]
struct CredentialsRequest<'a> {
    address: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
struct TokenResponse {
    token: String,
}

#[derive(Deserialize)]
struct AccountResource {
    id: String,
    address: String,
}

#[derive(Deserialize)]
struct Collection<T> {
    #[serde(rename = "hydra:member")]
    members: Vec<T>,
}

#[derive(Deserialize)]
struct DomainResource {
    domain: String,
    #[serde(rename = "isActive")]
    is_active: bool,
}

#[derive(Deserialize)]
struct MessageListResource {
    id: String,
    from: Address,
    subject: String,
    intro: String,
    seen: bool,
    #[serde(rename = "hasAttachments")]
    has_attachments: bool,
    size: u64,
    #[serde(rename = "createdAt")]
    created_at: String,
}

impl From<MessageListResource> for MessageSummary {
    fn from(value: MessageListResource) -> Self {
        Self {
            id: value.id,
            from: value.from,
            subject: value.subject,
            intro: value.intro,
            seen: value.seen,
            has_attachments: value.has_attachments,
            size: value.size,
            created_at: value.created_at,
        }
    }
}

#[derive(Deserialize)]
struct MessageResource {
    id: String,
    from: Address,
    subject: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    html: Vec<String>,
    seen: bool,
    #[serde(default)]
    attachments: Vec<AttachmentResource>,
    #[serde(rename = "createdAt")]
    created_at: String,
}

#[derive(Deserialize)]
struct AttachmentResource {
    id: String,
    filename: String,
    #[serde(rename = "contentType")]
    content_type: String,
    size: u64,
    #[serde(rename = "downloadUrl")]
    download_url: String,
}

impl From<AttachmentResource> for Attachment {
    fn from(value: AttachmentResource) -> Self {
        Self {
            id: value.id,
            filename: value.filename,
            content_type: value.content_type,
            size: value.size,
            download_url: value.download_url,
        }
    }
}

impl From<MessageResource> for Message {
    fn from(value: MessageResource) -> Self {
        Self {
            id: value.id,
            from: value.from,
            subject: value.subject,
            text: value.text,
            html: value.html,
            seen: value.seen,
            attachments: value.attachments.into_iter().map(Into::into).collect(),
            created_at: value.created_at,
        }
    }
}
