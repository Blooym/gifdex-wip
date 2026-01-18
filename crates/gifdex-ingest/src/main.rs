mod database;
mod handlers;

use crate::{
    database::Database,
    handlers::{
        actor::{handle_profile_create_event, handle_profile_delete_event},
        feed::{
            handle_favourite_create_event, handle_favourite_delete_event, handle_post_create,
            handle_post_delete,
        },
        handle_unknown_event,
        identity::handle_identity,
        moderation::{handle_label_create_event, handle_label_delete_event},
    },
};
use anyhow::Result;
use clap::Parser;
use dotenvy::dotenv;
use floodgate::{
    api::{EventData, RecordAction},
    client::TapClient,
};
use gifdex_lexicons::net_gifdex::{
    self,
    actor::profile::Profile,
    feed::{favourite::Favourite, post::Post},
    moderation::label::Label,
};
use jacquard_common::types::{collection::Collection, did::Did};
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
    tap_password: String,

    #[clap(
        long = "moderation-accounts",
        env = "LESGIF_INGEST_MODERATION_ACCOUNTS"
    )]
    moderation_account_dids: Vec<Did<'static>>,
}

#[derive(Clone)]
struct AppState {
    database: Database,
    moderation_account_dids: Vec<Did<'static>>,
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
    let state = Arc::new(AppState {
        database,
        moderation_account_dids: args.moderation_account_dids,
    });

    // Setup the tap client and ensure things are healthy.
    let tap_client = TapClient::builder(args.tap_url.clone())
        .password(&args.tap_password)
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

        match connection
            .handler(move |data| {
                let state = state.clone();
                handle_event(state, data)
            })
            .await
        {
            Ok(()) => {
                tracing::info!("Connection closed - reconnecting in 10 seconds");
            }
            Err(err) => {
                tracing::error!("Handler failed - retrying in 10 seconds: {err:?}");
            }
        }
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

#[tracing::instrument(
    skip(state, data),
    fields(
        event_type = match &data {
            EventData::Identity { .. } => "identity",
            EventData::Record { .. } => "record",
            _ => "unknown",
        },
        did = match &data {
            EventData::Identity { identity, .. } => Some(identity.did.as_str()),
            EventData::Record { record, .. } => Some(record.did.as_str()),
            _ => None,
        },
        handle = match &data {
            EventData::Identity { identity, .. } => Some(identity.handle.as_str()),
            _ => None,
        },
        status = match &data {
            EventData::Identity { identity, .. } => Some(identity.status.as_str()),
            _ => None,
        },
        is_active = match &data {
            EventData::Identity { identity, .. } => Some(identity.is_active),
            _ => None,
        },
        collection = match &data {
            EventData::Record { record, .. } => Some(record.collection.as_str()),
            _ => None,
        },
        rkey = match &data {
            EventData::Record { record, .. } => Some(record.rkey.as_str()),
            _ => None,
        },
        live = match &data {
            EventData::Record { record, .. } => Some(record.live),
            _ => None,
        },
        action = match &data {
            EventData::Record { record, .. } => Some(match &record.action {
                RecordAction::Create { .. } => "create",
                RecordAction::Update { .. } => "update",
                RecordAction::Delete => "delete",
            }),
            _ => None,
        },
    )
)]
async fn handle_event(state: Arc<AppState>, data: EventData<'static>) -> anyhow::Result<()> {
    match data {
        EventData::Identity { identity } => {
            handle_identity(&state, &identity).await;
            Ok(())
        }
        EventData::Record { record } => {
            // Check moderation account restriction
            if record.collection.starts_with("net.gifdex.moderation")
                && !state.moderation_account_dids.contains(&record.did)
            {
                tracing::warn!(
                    "Rejected record: moderation record from account not marked as an accepted moderation account"
                );
                return Ok(()); // Ack anyway
            }

            match &record.action {
                RecordAction::Create {
                    record: payload, ..
                }
                | RecordAction::Update {
                    record: payload, ..
                } => match record.collection.as_str() {
                    net_gifdex::feed::post::Post::NSID => {
                        let json_str = serde_json::to_string(&payload.raw())?;
                        let post: Post = serde_json::from_str(&json_str)?;
                        handle_post_create(&state, &record, &post).await
                    }

                    net_gifdex::feed::favourite::Favourite::NSID => {
                        let json_str = serde_json::to_string(&payload.raw())?;
                        let fav: Favourite = serde_json::from_str(&json_str)?;
                        handle_favourite_create_event(&state, &record, &fav).await
                    }

                    net_gifdex::actor::profile::Profile::NSID => {
                        let json_str = serde_json::to_string(&payload.raw())?;
                        let profile: Profile = serde_json::from_str(&json_str)?;
                        handle_profile_create_event(&state, &record, &profile).await
                    }

                    net_gifdex::moderation::label::Label::NSID => {
                        let json_str = serde_json::to_string(&payload.raw())?;
                        let label: Label = serde_json::from_str(&json_str)?;
                        handle_label_create_event(&state, &record, &label).await
                    }
                    _ => handle_unknown_event(&state, &record).await,
                },

                RecordAction::Delete => match record.collection.as_str() {
                    net_gifdex::feed::post::Post::NSID => handle_post_delete(&state, &record).await,
                    net_gifdex::feed::favourite::Favourite::NSID => {
                        handle_favourite_delete_event(&state, &record).await
                    }
                    net_gifdex::actor::profile::Profile::NSID => {
                        handle_profile_delete_event(&state, &record).await
                    }
                    net_gifdex::moderation::label::Label::NSID => {
                        handle_label_delete_event(&state, &record).await
                    }
                    _ => handle_unknown_event(&state, &record).await,
                },
            }
        }
        _ => {
            panic!("unknown record action");
        }
    }
}
