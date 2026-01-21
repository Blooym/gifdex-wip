mod identity;
mod net_gifdex;

use crate::AppState;
use crate::handlers;
use crate::handlers::net_gifdex::actor::handle_profile_create_event;
use crate::handlers::net_gifdex::actor::handle_profile_delete_event;
use crate::handlers::net_gifdex::feed::handle_favourite_create_event;
use crate::handlers::net_gifdex::feed::handle_favourite_delete_event;
use crate::handlers::net_gifdex::feed::handle_post_create;
use crate::handlers::net_gifdex::feed::handle_post_delete;
use crate::handlers::net_gifdex::labeler::handle_label_create_event;
use crate::handlers::net_gifdex::labeler::handle_label_delete_event;
use crate::handlers::net_gifdex::labeler::handle_rule_create_event;
use crate::handlers::net_gifdex::labeler::handle_rule_delete_event;
use anyhow::bail;
use floodgate::api::{EventData, RecordAction};
use gifdex_lexicons::net_gifdex as gifdex_lexicons;
use jacquard_common::types::collection::Collection;
use sqlx::query;
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
        rev = match &data {
            EventData::Record { record, .. } => Some(record.rev.as_str()),
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
        EventData::Identity { identity } => {
            let mut tx = state.database.transaction().await?;
            handlers::identity::handle_identity(&identity, &mut tx, &state).await?;
            tx.commit().await?;
            Ok(())
        }
        EventData::Record { record } => {
            let mut tx = state.database.transaction().await?;
            match &record.action {
                RecordAction::Create {
                    record: payload, ..
                }
                | RecordAction::Update {
                    record: payload, ..
                } => match record.collection.as_str() {
                    gifdex_lexicons::feed::post::Post::NSID => {
                        let json_str = serde_json::to_string(&payload.raw())?;
                        let post: gifdex_lexicons::feed::post::Post =
                            serde_json::from_str(&json_str)?;
                        handle_post_create(&record, &post, &mut tx, &state).await?
                    }
                    gifdex_lexicons::feed::favourite::Favourite::NSID => {
                        let json_str = serde_json::to_string(&payload.raw())?;
                        let favourite: gifdex_lexicons::feed::favourite::Favourite =
                            serde_json::from_str(&json_str)?;
                        handle_favourite_create_event(&record, &favourite, &mut tx, &state).await?
                    }
                    gifdex_lexicons::actor::profile::Profile::NSID => {
                        let json_str = serde_json::to_string(&payload.raw())?;
                        let profile: gifdex_lexicons::actor::profile::Profile =
                            serde_json::from_str(&json_str)?;
                        handle_profile_create_event(&record, &profile, &mut tx, &state).await?
                    }
                    gifdex_lexicons::labeler::label::Label::NSID => {
                        let json_str = serde_json::to_string(&payload.raw())?;
                        let label: gifdex_lexicons::labeler::label::Label =
                            serde_json::from_str(&json_str)?;
                        handle_label_create_event(&record, &label, &mut tx, &state).await?
                    }
                    gifdex_lexicons::labeler::rule::Rule::NSID => {
                        let json_str = serde_json::to_string(&payload.raw())?;
                        let rule: gifdex_lexicons::labeler::rule::Rule =
                            serde_json::from_str(&json_str)?;
                        handle_rule_create_event(&record, &rule, &mut tx, &state).await?
                    }
                    collection @ _ => {
                        tracing::error!(
                            "No record create/update handler for collection {collection} - please ensure tap is sending the correct records."
                        );
                        bail!("No registered create/update handler for record");
                    }
                },

                RecordAction::Delete => match record.collection.as_str() {
                    gifdex_lexicons::feed::post::Post::NSID => {
                        handle_post_delete(&record, &mut tx, &state).await?
                    }
                    gifdex_lexicons::feed::favourite::Favourite::NSID => {
                        handle_favourite_delete_event(&record, &mut tx, &state).await?
                    }
                    gifdex_lexicons::actor::profile::Profile::NSID => {
                        handle_profile_delete_event(&record, &mut tx, &state).await?
                    }
                    gifdex_lexicons::labeler::label::Label::NSID => {
                        handle_label_delete_event(&record, &mut tx, &state).await?
                    }
                    gifdex_lexicons::labeler::rule::Rule::NSID => {
                        handle_rule_delete_event(&record, &mut tx, &state).await?
                    }
                    collection @ _ => {
                        tracing::error!(
                            "No record delete handler for collection {collection} - please ensure tap is sending the correct records."
                        );
                        bail!("No registered delete handler for record");
                    }
                },
            }

            // Update repository revision.
            tracing::debug!("updated repository revision to {}", record.rev);
            query!(
                "UPDATE accounts SET rev = $2 WHERE did = $1",
                record.did.as_str(),
                record.rev.as_str(),
            )
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;

            Ok(())
        }
        etype @ _ => {
            panic!("unknown event data type: {etype:?}");
        }
    }
}
