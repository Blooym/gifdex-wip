mod identity;
mod net_gifdex;

use crate::AppState;
use crate::handlers::{
    self,
    net_gifdex::{
        actor::{handle_profile_create_event, handle_profile_delete_event},
        feed::{
            handle_favourite_create_event, handle_favourite_delete_event, handle_post_create,
            handle_post_delete,
        },
        labeler::{
            handle_label_create_event, handle_label_delete_event, handle_rule_create_event,
            handle_rule_delete_event,
        },
    },
};
use anyhow::bail;
use doubletap::types::{EventData, RecordAction};
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
                _ => "unknown",
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
                        handle_post_create(&record, &payload.deserialize()?, &mut tx, &state)
                            .await?
                    }
                    gifdex_lexicons::feed::favourite::Favourite::NSID => {
                        handle_favourite_create_event(
                            &record,
                            &payload.deserialize()?,
                            &mut tx,
                            &state,
                        )
                        .await?
                    }
                    gifdex_lexicons::actor::profile::Profile::NSID => {
                        handle_profile_create_event(
                            &record,
                            &payload.deserialize()?,
                            &mut tx,
                            &state,
                        )
                        .await?
                    }
                    gifdex_lexicons::labeler::label::Label::NSID => {
                        handle_label_create_event(&record, &payload.deserialize()?, &mut tx, &state)
                            .await?
                    }
                    gifdex_lexicons::labeler::rule::Rule::NSID => {
                        handle_rule_create_event(&record, &payload.deserialize()?, &mut tx, &state)
                            .await?
                    }
                    collection @ _ => {
                        tracing::error!(
                            "No record create/update handler for collection '{collection}': please ensure tap is sending the correct collections."
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
                    collection => {
                        tracing::error!(
                            "No record delete handler for collection '{collection}': please ensure tap is sending the correct collections."
                        );
                        bail!("No registered delete handler for record");
                    }
                },
                operation => {
                    tracing::error!("Unknown operation '{operation:?}'");
                    bail!("No registered handlers for operation {operation:?}");
                }
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
        event_type => {
            tracing::error!("Unknown event data type '{event_type:?}");
            bail!("unknown event data type: {event_type:?}");
        }
    }
}
