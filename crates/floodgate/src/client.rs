use crate::{
    api::{
        CursorsResponse, OutboxBufferResponse, RecordCountResponse, RepoCountResponse, RepoInfo,
        ResyncBufferResponse,
    },
    channel::{Channel, ChannelBuilder},
};
use base64::Engine;
use jacquard_common::{
    IntoStatic,
    types::{did::Did, did_doc::DidDocument},
};
use reqwest::{
    Response,
    header::{AUTHORIZATION, HeaderMap, HeaderValue, InvalidHeaderValue},
};
use serde::Serialize;
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone)]
#[must_use]
pub struct TapClient {
    http_client: reqwest::Client,
    base_url: Url,
    password: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum TapRequestError {
    #[error("network request failed due to: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("tap responded with an error: {0:?}")]
    ErrorResponse(Response),
    #[error("server responded with an invalid response. failed to deserialise due to {0}")]
    InvalidResponseBody(#[from] serde_json::Error),
}

impl TapClient {
    pub fn new(base_url: Url) -> Result<Self, TapClientBuildError> {
        Self::builder(base_url).build()
    }

    pub fn builder(base_url: Url) -> TapClientBuilder {
        TapClientBuilder {
            base_url,
            password: None,
        }
    }

    pub fn url(&self) -> &Url {
        &self.base_url
    }

    pub async fn health(&self) -> Result<(), TapRequestError> {
        log::debug!("fetching tap health status");
        let response = self
            .http_client
            .get(self.base_url.join("/health").expect(
                "constructing the endpoint url from the base url should always be possible",
            ))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(TapRequestError::ErrorResponse(response));
        }
        Ok(())
    }

    pub async fn resolve_did(
        &self,
        did: &Did<'_>,
    ) -> Result<DidDocument<'static>, TapRequestError> {
        log::debug!("resolving {did}");
        let response = self
            .http_client
            .get(self.base_url.join(&format!("/resolve/{did}")).expect(
                "constructing the endpoint url from the base url should always be possible",
            ))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(TapRequestError::ErrorResponse(response));
        }
        let bytes = response.bytes().await?;
        let data: DidDocument = serde_json::from_slice(&bytes)?;
        Ok(data.into_static())
    }

    pub async fn repo_info(&self, did: &Did<'_>) -> Result<RepoInfo<'static>, TapRequestError> {
        log::debug!("fetching repo information for {did}");
        let response = self
            .http_client
            .get(self.base_url.join(&format!("/info/{did}")).expect(
                "constructing the endpoint url from the base url should always be possible",
            ))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(TapRequestError::ErrorResponse(response));
        }
        let bytes = response.bytes().await?;
        let data: RepoInfo = serde_json::from_slice(&bytes)?;
        Ok(data.into_static())
    }

    pub async fn add_repos(&self, dids: &[Did<'_>]) -> Result<(), TapRequestError> {
        log::debug!("adding {dids:?} to tap's tracked repositories");
        #[derive(Serialize)]
        struct Payload<'a> {
            dids: &'a [Did<'a>],
        }
        let payload = Payload { dids };
        let response = self
            .http_client
            .post(self.base_url.join("/repos/add").expect(
                "constructing the endpoint url from the base url should always be possible",
            ))
            .json(&payload)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(TapRequestError::ErrorResponse(response));
        }
        Ok(())
    }

    pub async fn remove_repos(&self, dids: &[Did<'_>]) -> Result<(), TapRequestError> {
        log::debug!("removing {dids:?} from tap's tracked repositories");
        #[derive(Serialize)]
        struct Payload<'a> {
            dids: &'a [Did<'a>],
        }
        let payload = Payload { dids };
        let response = self
            .http_client
            .post(self.base_url.join("/repos/remove").expect(
                "constructing the endpoint url from the base url should always be possible",
            ))
            .json(&payload)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(TapRequestError::ErrorResponse(response));
        }
        Ok(())
    }

    pub async fn repo_count(&self) -> Result<RepoCountResponse, TapRequestError> {
        log::debug!("fetching tap tracked repository count");
        let response = self
            .http_client
            .get(self.base_url.join("/stats/repo-count").expect(
                "constructing the endpoint url from the base url should always be possible",
            ))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(TapRequestError::ErrorResponse(response));
        }
        let data = response.json::<RepoCountResponse>().await?;
        Ok(data)
    }

    pub async fn record_count(&self) -> Result<RecordCountResponse, TapRequestError> {
        log::debug!("fetching tap tracked record count");
        let response = self
            .http_client
            .get(self.base_url.join("/stats/record-count").expect(
                "constructing the endpoint url from the base url should always be possible",
            ))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(TapRequestError::ErrorResponse(response));
        }
        let data = response.json::<RecordCountResponse>().await?;
        Ok(data)
    }

    pub async fn outbox_buffer(&self) -> Result<OutboxBufferResponse, TapRequestError> {
        log::debug!("fetching event count in tap outbox buffer");
        let response = self
            .http_client
            .get(self.base_url.join("/stats/outbox-buffer").expect(
                "constructing the endpoint url from the base url should always be possible",
            ))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(TapRequestError::ErrorResponse(response));
        }
        let data = response.json::<OutboxBufferResponse>().await?;
        Ok(data)
    }

    pub async fn resync_buffer(&self) -> Result<ResyncBufferResponse, TapRequestError> {
        log::debug!("fetching event count in tap resync buffer");
        let response = self
            .http_client
            .get(self.base_url.join("/stats/resync-buffer").expect(
                "constructing the endpoint url from the base url should always be possible",
            ))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(TapRequestError::ErrorResponse(response));
        }
        let data = response.json::<ResyncBufferResponse>().await?;
        Ok(data)
    }

    pub async fn cursors(&self) -> Result<CursorsResponse, TapRequestError> {
        log::debug!("getting tap's current cursor positions");
        let url = self
            .base_url
            .join("/stats/cursors")
            .expect("constructing the endpoint url from the base url should always be possible");
        let response = self.http_client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(TapRequestError::ErrorResponse(response));
        }
        let data = response.json::<CursorsResponse>().await?;
        Ok(data)
    }

    /// Create a channel for connecting to the event stream
    ///
    /// Note: This carries the password from the TapClient if one was set.
    pub fn channel(&self) -> ChannelBuilder {
        let mut builder = Channel::builder(self.base_url.clone());
        if let Some(ref password) = self.password {
            builder = builder.password(password.clone());
        }
        builder
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
#[must_use]
pub struct TapClientBuilder {
    base_url: Url,
    password: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum TapClientBuildError {
    #[error("invalid url scheme: {0}. must be either http or https")]
    InvalidUrlScheme(String),
    #[error("invalid password. failed to convert password to an authorization header due to: {0}")]
    InvalidPassword(InvalidHeaderValue),
    #[error("failed to construct the internal http client due to: {0}")]
    CouldntConstructHttpClient(#[from] reqwest::Error),
}

impl TapClientBuilder {
    pub fn password<P: Into<String>>(mut self, password: Option<P>) -> Self {
        self.password = password.map(|p| p.into());
        self
    }

    pub fn build(self) -> Result<TapClient, TapClientBuildError> {
        if !matches!(self.base_url.scheme(), "http" | "https") {
            return Err(TapClientBuildError::InvalidUrlScheme(
                self.base_url.scheme().into(),
            ));
        }

        let mut headers = HeaderMap::new();

        // Add authorization header if password is provided
        if let Some(ref password) = self.password {
            let encoded =
                base64::engine::general_purpose::STANDARD.encode(format!("admin:{password}"));
            let auth_value = HeaderValue::from_str(&format!("Basic {encoded}"))
                .map_err(|err| TapClientBuildError::InvalidPassword(err))?;
            headers.insert(AUTHORIZATION, auth_value);
        }

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .default_headers(headers)
            .build()?;

        Ok(TapClient {
            http_client,
            base_url: self.base_url,
            password: self.password,
        })
    }
}
