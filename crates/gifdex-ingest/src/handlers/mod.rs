mod actor;
mod feed;
mod identity;
mod labeler;

use crate::AppState;
use crate::handlers::labeler::{handle_rule_create_event, handle_rule_delete_event};
use crate::handlers::{
    actor::{handle_profile_create_event, handle_profile_delete_event},
    feed::{
        handle_favourite_create_event, handle_favourite_delete_event, handle_post_create,
        handle_post_delete,
    },
    identity::handle_identity,
    labeler::{handle_label_create_event, handle_label_delete_event},
};
use anyhow::bail;
use floodgate::api::{EventData, RecordAction};
use gifdex_lexicons::net_gifdex;
use jacquard_common::types::collection::Collection;
use std::sync::Arc;

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
pub async fn handle_event(state: Arc<AppState>, data: EventData<'static>) -> anyhow::Result<()> {
    match data {
        EventData::Identity { identity } => handle_identity(&state, &identity).await,
        EventData::Record { record } => match &record.action {
            RecordAction::Create {
                record: payload, ..
            }
            | RecordAction::Update {
                record: payload, ..
            } => match record.collection.as_str() {
                net_gifdex::feed::post::Post::NSID => {
                    let json_str = serde_json::to_string(&payload.raw())?;
                    let post: net_gifdex::feed::post::Post = serde_json::from_str(&json_str)?;
                    handle_post_create(&state, &record, &post).await
                }
                net_gifdex::feed::favourite::Favourite::NSID => {
                    let json_str = serde_json::to_string(&payload.raw())?;
                    let favourite: net_gifdex::feed::favourite::Favourite =
                        serde_json::from_str(&json_str)?;
                    handle_favourite_create_event(&state, &record, &favourite).await
                }
                net_gifdex::actor::profile::Profile::NSID => {
                    let json_str = serde_json::to_string(&payload.raw())?;
                    let profile: net_gifdex::actor::profile::Profile =
                        serde_json::from_str(&json_str)?;
                    handle_profile_create_event(&state, &record, &profile).await
                }
                net_gifdex::labeler::label::Label::NSID => {
                    let json_str = serde_json::to_string(&payload.raw())?;
                    let label: net_gifdex::labeler::label::Label = serde_json::from_str(&json_str)?;
                    handle_label_create_event(&state, &record, &label).await
                }
                net_gifdex::labeler::rule::Rule::NSID => {
                    let json_str = serde_json::to_string(&payload.raw())?;
                    let rule: net_gifdex::labeler::rule::Rule = serde_json::from_str(&json_str)?;
                    handle_rule_create_event(&state, &record, &rule).await
                }
                collection @ _ => {
                    tracing::error!(
                        "No record create/update handler for collection {collection} - please ensure tap is sending the correct records."
                    );
                    bail!("No registered create/update handler for record");
                }
            },

            RecordAction::Delete => match record.collection.as_str() {
                net_gifdex::feed::post::Post::NSID => handle_post_delete(&state, &record).await,
                net_gifdex::feed::favourite::Favourite::NSID => {
                    handle_favourite_delete_event(&state, &record).await
                }
                net_gifdex::actor::profile::Profile::NSID => {
                    handle_profile_delete_event(&state, &record).await
                }
                net_gifdex::labeler::label::Label::NSID => {
                    handle_label_delete_event(&state, &record).await
                }
                net_gifdex::labeler::rule::Rule::NSID => {
                    handle_rule_delete_event(&state, &record).await
                }
                collection @ _ => {
                    tracing::error!(
                        "No record delete handler for collection {collection} - please ensure tap is sending the correct records."
                    );
                    bail!("No registered delete handler for record");
                }
            },
        },
        etype @ _ => {
            panic!("unknown event data type: {etype:?}");
        }
    }
}
