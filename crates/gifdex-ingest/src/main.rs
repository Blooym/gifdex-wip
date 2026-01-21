mod database;
mod handlers;

use crate::{database::Database, handlers::handle_event};
use anyhow::{Context, Result};
use clap::Parser;
use dotenvy::dotenv;
use floodgate::client::TapClient;
use std::{num::NonZero, sync::Arc, time::Duration};
use tracing_subscriber::EnvFilter;
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about)]
struct Arguments {
    /// The local database URL to use for persistent storage.
    #[clap(long = "database-url", env = "DATABASE_URL")]
    database_url: String,

    #[clap(long = "tap-url", env = "GIFDEX_INGEST_TAP_URL")]
    tap_url: Url,

    #[clap(long = "tap-password", env = "GIFDEX_INGEST_TAP_PASSWORD")]
    tap_password: Option<String>,

    #[clap(long = "concurrent-messages", env = "GIFDEX_CONCURRENT_MESSAGES")]
    concurrent_messages: NonZero<usize>,
}

struct AppState {
    database: Database,
    tap_client: TapClient,
    http_client: reqwest::Client,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install default rustls crypto provider");
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info")))
        .init();
    let args = Arguments::parse();

    // Initialise application state.
    let tap_client = TapClient::builder(args.tap_url.clone())
        .password(args.tap_password)
        .build()
        .context("failed to initialise tap client")?;
    let tap_channel = tap_client
        .channel()
        .max_concurrent(args.concurrent_messages)
        .build()
        .context("failed to construct tap channel")?;
    let http_client = reqwest::Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .context("failed to initialise http client")?;

    let database = Database::new(&args.database_url)
        .await
        .context("failed to initialise database")?;

    // Connect to tap and begin processing events.
    let state = Arc::new(AppState {
        database,
        tap_client,
        http_client,
    });
    loop {
        const TAP_RECONNECT_INTERVAL: Duration = Duration::from_secs(30);
        let state = state.clone();
        let connection = match tap_channel.connect().await {
            Ok(r) => r,
            Err(err) => {
                tracing::error!(
                    "Unable to connect to tap channel - retrying in {TAP_RECONNECT_INTERVAL:?}: {err:?}"
                );
                tokio::time::sleep(TAP_RECONNECT_INTERVAL).await;
                continue;
            }
        };
        connection
            .handler(move |data| {
                let state = state.clone();
                handle_event(state, data)
            })
            .await;
        tracing::info!(
            "Tap channel was closed while handling events -  reconnecting automatically in {TAP_RECONNECT_INTERVAL:?}: "
        );
        tokio::time::sleep(TAP_RECONNECT_INTERVAL).await;
    }
}
