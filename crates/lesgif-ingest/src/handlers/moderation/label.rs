use crate::{AppState, tap::TapRecordEventData};
use chrono::DateTime;
use lesgif_lexicons::net_dollware::lesgif::moderation::label::Label;
use sqlx::query;
use tracing::{error, info};

pub async fn handle_label_create_event(
    state: &AppState,
    record_data: &TapRecordEventData<'_>,
    data: &Label<'_>,
) -> bool {
    // https://tangled.org/nonbinary.computer/jacquard/issues/27
    match query!(
        "INSERT INTO labels (subject, rkey, value, reason, actor, \
         expires_at, created_at, ingested_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, \
         extract(epoch from now())::BIGINT) \
         ON CONFLICT(subject, rkey) DO UPDATE SET \
         reason = excluded.reason, \
         actor = excluded.actor, \
         value = excluded.value, \
         expires_at = excluded.expires_at",
        data.subject.as_str(),
        record_data.rkey.as_str(),
        data.value.as_str(),
        data.reason.as_deref(),
        record_data.did.as_str(),
        data.expires_at.as_ref().map(|d| {
            DateTime::parse_from_rfc3339(d.as_str())
                .unwrap()
                .timestamp()
        }),
        DateTime::parse_from_rfc3339(data.created_at.as_str())
            .unwrap()
            .timestamp()
    )
    .execute(state.database.executor())
    .await
    {
        Ok(_) => {
            info!("Upserted moderation label into database");
            true
        }
        Err(err) => {
            error!("Failed to upsert moderation label into database: {err:?}");
            false
        }
    }
}

pub async fn handle_label_delete_event(
    state: &AppState,
    record_data: &TapRecordEventData<'_>,
) -> bool {
    match query!(
        "DELETE FROM labels WHERE rkey = $1",
        record_data.rkey.as_str()
    )
    .execute(state.database.executor())
    .await
    {
        Ok(_) => {
            info!("Deleted moderation label from database");
            true
        }
        Err(err) => {
            error!("Failed to delete moderation label from database: {err:?}");
            false
        }
    }
}
