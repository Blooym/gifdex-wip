mod database;
mod handlers;
mod tap;

use crate::{database::Database, tap::run_tap_consumer};
use anyhow::Result;
use clap::Parser;
use dotenvy::dotenv;
use jacquard_common::types::did::Did;
use std::sync::Arc;
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
    tap_password: String,

    #[clap(long = "moderation-account", env = "LESGIF_INGEST_MODERATION_ACCOUNT")]
    moderation_account_did: Did<'static>,
}

#[derive(Clone)]
struct AppState {
    database: Database,
    moderation_account_did: Did<'static>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info")))
        .init();
    let args = Arguments::parse();
    let database = Database::new(&args.database_url).await?;
    let state = Arc::new(AppState {
        database,
        moderation_account_did: args.moderation_account_did,
    });
    run_tap_consumer(state, &args.tap_url, &args.tap_password).await;
    Ok(())
}
