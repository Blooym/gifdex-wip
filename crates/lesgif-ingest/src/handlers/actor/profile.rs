use crate::{AppState, tap::TapRecordEventData};
use lesgif_lexicons::net_dollware::lesgif::actor::profile::Profile;
use sqlx::query;
use tracing::{error, info, warn};

pub async fn handle_profile_create_event(
    state: &AppState,
    record_data: &TapRecordEventData<'_>,
    data: &Profile<'_>,
) -> bool {
    if record_data.rkey.as_str() != "self" {
        warn!(
            "Rejected record: actor profile record is invalid as it does not use the rkey 'self'"
        );
        return true;
    }
    match query!(
        "INSERT INTO accounts (did, display_name, description, pronouns, \
                             avatar_blob_cid) \
                             VALUES ($1, $2, $3, $4, $5) \
                             ON CONFLICT(did) DO UPDATE SET \
                             display_name = excluded.display_name, \
                             description = excluded.description, \
                             pronouns = excluded.pronouns, \
                             avatar_blob_cid = excluded.avatar_blob_cid",
        record_data.did.as_str(),
        data.display_name.as_deref(),
        data.description.as_deref(),
        data.pronouns.as_deref(),
        data.avatar.as_ref().map(|s| s.blob().cid().as_str())
    )
    .execute(state.database.executor())
    .await
    {
        Ok(_) => {
            info!("Upserted user-defined actor profile fields into database");
            true
        }
        Err(err) => {
            error!("Failed to upsert user-defined actor profile fields into database: {err:?}");
            false
        }
    }
}

pub async fn handle_profile_delete_event(
    state: &AppState,
    record_data: &TapRecordEventData<'_>,
) -> bool {
    match query!(
        "UPDATE accounts SET \
                             display_name = NULL, \
                             description = NULL, \
                             pronouns = NULL, \
                             avatar_blob_cid = NULL \
                             WHERE did = $1",
        record_data.did.as_str()
    )
    .execute(state.database.executor())
    .await
    {
        Ok(_) => {
            info!("Cleared all user-defined actor profile fields from database");
            true
        }
        Err(err) => {
            error!("Failed to clear user-defined actor profile fields from database: {err:?}");
            false
        }
    }
}
