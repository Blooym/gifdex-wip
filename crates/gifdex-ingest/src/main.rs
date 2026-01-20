mod database;
mod handlers;

use crate::{database::Database, handlers::handle_event};
use anyhow::Result;
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

    #[clap(long = "tap-url", env = "LESGIF_INGEST_TAP_URL")]
    tap_url: Url,

    #[clap(long = "tap-password", env = "LESGIF_INGEST_TAP_PASSWORD")]
    tap_password: Option<String>,
}

#[derive(Clone)]
struct AppState {
    database: Database,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info")))
        .init();
    let args = Arguments::parse();

    // Required - see https://github.com/snapview/tokio-tungstenite/issues/353 for details.
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install default rustls crypto provider");

    // Connect to database and initialise state.
    let database = Database::new(&args.database_url).await?;
    let state = Arc::new(AppState { database });

    // Setup the tap client and ensure things are healthy.
    let tap_client = TapClient::builder(args.tap_url.clone())
        .password(args.tap_password)
        .build()?;
    let channel = tap_client
        .channel()
        .max_concurrent(NonZero::new(50).unwrap())
        .build()?;
    loop {
        let state = state.clone();

        let connection = match channel.connect().await {
            Ok(r) => r,
            Err(err) => {
                tracing::error!("Connection failed - retrying in 5 seconds: {err:?}");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        connection
            .handler(move |data| {
                let state = state.clone();
                handle_event(state, data)
            })
            .await;
        tracing::info!("Connection closed - reconnecting in 10 seconds");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
