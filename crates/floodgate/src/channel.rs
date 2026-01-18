use anyhow::{Context, Result, bail};
use futures_util::{SinkExt, StreamExt};
use jacquard_common::IntoStatic;
use reqwest::header::{AUTHORIZATION, HeaderValue, USER_AGENT};
use serde::Serialize;
use std::{num::NonZero, sync::Arc};
use tokio::{
    sync::{Semaphore, mpsc},
    task::JoinSet,
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};
use url::Url;

use crate::api::{Event, EventData};

// TODO: Cleanup structs in here to put them in better places?
// TODO: Implement graceful shutdown.
// TODO: Consider blocking tasks when sending acks to ensure they're sent before taking on new work?
// TODO: Better error handling.

#[derive(Debug)]
#[must_use]
#[non_exhaustive]
pub struct ChannelConnectionHandle {
    read: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    ack_tx: mpsc::UnboundedSender<u64>,
    semaphore: Arc<Semaphore>,
}

impl ChannelConnectionHandle {
    pub(crate) async fn connect(
        base_url: Url,
        password: Option<&str>,
        max_concurrent: NonZero<usize>,
    ) -> Result<Self> {
        let mut websocket_url = base_url.clone();
        match websocket_url.scheme() {
            "http" => websocket_url.set_scheme("ws").unwrap(),
            "https" => websocket_url.set_scheme("wss").unwrap(),
            "ws" | "wss" => {}
            _ => bail!("Invalid URL Scheme: {}", websocket_url.scheme()),
        }
        websocket_url.set_path("/channel");

        // Create websocket request with appropriate headers.
        let mut request = websocket_url.as_str().into_client_request()?;
        request.headers_mut().insert(
            USER_AGENT,
            HeaderValue::from_static(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            )),
        );
        if let Some(password) = password {
            use base64::Engine;
            let encoded =
                base64::engine::general_purpose::STANDARD.encode(format!("admin:{password}"));
            request.headers_mut().insert(
                AUTHORIZATION,
                format!("Basic {encoded}")
                    .parse()
                    .context("failed to create authorization header")?,
            );
        }

        // Open connection.
        log::debug!("opening websocket connection to {websocket_url}");
        let (websocket_stream, response) = connect_async(request)
            .await
            .context("failed to connect to websocket")?;
        if response.status().as_u16() != 101 && !response.status().is_success() {
            bail!(
                "websocket connection return unsuccessful response: {}",
                response.status()
            );
        }

        // Open appropriate channels for communicating via websocket.
        log::debug!("opened websocket connection to {websocket_url}");
        let (write, read) = websocket_stream.split();
        let (ack_tx, ack_rx) = mpsc::unbounded_channel::<u64>();
        let semaphore = Arc::new(Semaphore::new(max_concurrent.get()));

        log::trace!("spawning handler writer task");
        tokio::spawn(async move {
            Self::writer_task(write, ack_rx).await;
        });

        Ok(Self {
            read,
            ack_tx,
            semaphore,
        })
    }

    pub async fn handler<
        Handler: Fn(EventData<'static>) -> HandlerResult + Send + Sync + 'static,
        HandlerResult: std::future::Future<Output = Result<()>> + Send,
    >(
        mut self,
        handler: Handler,
    ) -> Result<()> {
        let handler = Arc::new(handler);
        let mut tasks = JoinSet::new();
        loop {
            // Remove any finished tasks in the set and log a panic if needed.
            while let Some(result) = tasks.try_join_next() {
                if let Err(err) = result {
                    if err.is_panic() {
                        log::error!("handler task panicked: {err:?}");
                    }
                }
            }
            let permit = match self.semaphore.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => break,
            };
            let message = match self.read.next().await {
                Some(msg) => msg,
                None => {
                    drop(permit);
                    break;
                }
            };
            match message {
                Ok(Message::Text(text)) => {
                    let ack_tx = self.ack_tx.clone();
                    let handler = handler.clone();
                    tasks.spawn(async move {
                        let event = match serde_json::from_str::<Event>(&text) {
                            Ok(e) => e.into_static(),
                            Err(err) => {
                                log::warn!("failed to parse event: {err:?}");
                                drop(permit);
                                return;
                            }
                        };
                        let result = handler(event.data).await;
                        if result.is_ok() {
                            if let Err(err) = ack_tx.send(event.id) {
                                log::warn!("failed to queue ack for event {}: {err:?}", event.id);
                            }
                        } else if let Err(err) = result {
                            log::warn!("event {} handler failed: {err:?}", event.id);
                        }
                        drop(permit);
                    });
                }
                Ok(Message::Close(_)) => {
                    log::info!("websocket closed");
                    drop(permit);
                    break;
                }
                Ok(_) => {
                    drop(permit);
                }
                Err(err) => {
                    log::error!("websocket error: {err:?}");
                    drop(permit);
                    break;
                }
            }
        }

        // Wait for all currently running handler tasks to run to completion.
        while let Some(result) = tasks.join_next().await {
            if let Err(err) = result {
                if err.is_panic() {
                    log::error!("handler task panicked: {err:?}");
                }
            }
        }

        Ok(())
    }

    async fn writer_task(
        mut write: futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
        mut ack_rx: mpsc::UnboundedReceiver<u64>,
    ) {
        #[derive(Serialize)]
        struct Ack {
            #[serde(rename = "type")]
            type_: &'static str,
            id: u64,
        }
        while let Some(id) = ack_rx.recv().await {
            let msg = Ack { type_: "ack", id };
            let json = match serde_json::to_string(&msg) {
                Ok(json) => json,
                Err(err) => {
                    log::warn!("failed to serialize ack: {err:?}");
                    continue;
                }
            };

            if let Err(err) = write.send(Message::Text(json.into())).await {
                log::warn!("failed to send ack: {err:?}");
                break;
            }
        }
    }
}

/// Configuration for a channel connection
#[derive(Debug, Clone)]
#[non_exhaustive]
#[must_use]
pub struct Channel {
    base_url: Url,
    password: Option<String>,
    max_concurrent: NonZero<usize>,
}

impl Channel {
    /// Create a new channel builder
    pub fn builder(base_url: Url) -> ChannelBuilder {
        ChannelBuilder::new(base_url)
    }

    /// Connect to the channel and return a ChannelReceiver
    pub async fn connect(&self) -> Result<ChannelConnectionHandle> {
        ChannelConnectionHandle::connect(
            self.base_url.clone(),
            self.password.as_deref(),
            self.max_concurrent,
        )
        .await
    }
}

/// Builder for creating a channel configuration
#[derive(Debug, Clone)]
#[non_exhaustive]
#[must_use]
pub struct ChannelBuilder {
    base_url: Url,
    password: Option<String>,
    max_concurrent: NonZero<usize>,
}

impl ChannelBuilder {
    /// Create a new channel builder with the given base URL
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            password: None,
            max_concurrent: NonZero::new(100).unwrap(),
        }
    }

    /// Set the password for authentication
    pub fn password<P: Into<String>>(mut self, password: P) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set the maximum number of concurrent handler tasks
    pub fn max_concurrent(mut self, max: NonZero<usize>) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Build and validate the channel configuration
    pub fn build(self) -> Result<Channel> {
        // Validate the URL scheme
        if !matches!(self.base_url.scheme(), "http" | "https" | "ws" | "wss") {
            return Err(anyhow::anyhow!(
                "Invalid URL scheme: {}. Must be http, https, ws, or wss",
                self.base_url.scheme()
            ));
        }

        Ok(Channel {
            base_url: self.base_url,
            password: self.password,
            max_concurrent: self.max_concurrent,
        })
    }
}
