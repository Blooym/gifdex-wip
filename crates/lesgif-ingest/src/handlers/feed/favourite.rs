use crate::{AppState, tap::TapRecordEventData};
use chrono::DateTime;
use lesgif_lexicons::net_dollware::lesgif::feed::favourite::Favourite;
use sqlx::query;
use tracing::{error, info};

pub async fn handle_favourite_create_event(
    state: &AppState,
    record_data: &TapRecordEventData<'_>,
    data: &Favourite<'_>,
) -> bool {
    // https://tangled.org/nonbinary.computer/jacquard/issues/27
    match query!(
        "INSERT INTO post_favourites (did, rkey, post_did, \
         post_rkey, created_at, ingested_at) \
         VALUES ($1, $2, $3, $4, $5, extract(epoch from now())::BIGINT) \
         ON CONFLICT (did, rkey) DO NOTHING",
        record_data.did.as_str(),
        record_data.rkey.as_str(),
        data.subject.authority().as_str(),
        data.subject.rkey().unwrap().0.as_str(),
        DateTime::parse_from_rfc3339(data.created_at.as_str())
            .unwrap()
            .timestamp()
    )
    .execute(state.database.executor())
    .await
    {
        Ok(_) => {
            info!("Upserted feed post favourite into database");
            true
        }
        Err(err) => {
            error!("Failed to upsert feed post favourite into database: {err:?}");
            false
        }
    }
}

pub async fn handle_favourite_delete_event(
    state: &AppState,
    record_data: &TapRecordEventData<'_>,
) -> bool {
    match query!(
        "DELETE FROM post_favourites WHERE did = $1 AND rkey = $2",
        record_data.did.as_str(),
        record_data.rkey.as_str()
    )
    .execute(state.database.executor())
    .await
    {
        Ok(_) => {
            info!("Deleted post favourite from database");
            true
        }
        Err(err) => {
            error!("Failed to delete post favourite from database: {err:?}");
            false
        }
    }
}
