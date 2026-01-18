use anyhow::Result;
use chrono::DateTime;
use floodgate::api::RecordEventData;
use gifdex_lexicons::net_gifdex;
use jacquard_common::types::{cid::Cid, tid::Tid};
use sqlx::query;
use tracing::{error, info, warn};

use crate::AppState;

pub async fn handle_post_create(
    state: &AppState,
    record_data: &RecordEventData<'_>,
    data: &net_gifdex::feed::post::Post<'_>,
) -> Result<()> {
    // Validate rkey format as tid:cid and matches blob
    match record_data.rkey.split_once(":") {
        Some((tid_str, cid_str)) => {
            if Tid::new(tid_str).is_err() {
                warn!("Rejected record: invalid TID in rkey");
                return Ok(());
            }
            let cid = match Cid::new(cid_str.as_bytes()) {
                Ok(cid) => cid,
                Err(_) => {
                    warn!("Rejected record: invalid CID in rkey");
                    return Ok(());
                }
            };
            // Validate rkey CID matches blob CID
            if cid != *data.gif.blob.blob().cid() {
                warn!("Rejected record: rkey CID doesn't match blob CID");
                return Ok(());
            }
        }
        None => {
            warn!("Rejected record: rkey doesn't match tid:cid format");
            return Ok(());
        }
    };

    // Validate the provided blob at least declared to be a gif.
    if data.gif.blob.blob().mime_type.as_str() != "image/gif" {
        warn!("Rejected record: blob isn't of mimeType 'image/gif'");
        return Ok(());
    }

    let tags_array: Option<Vec<String>> = if data.tags.is_empty() {
        None
    } else {
        Some(data.tags.iter().map(|cow| cow.to_string()).collect())
    };
    let languages_array: Option<Vec<String>> = data
        .languages
        .as_ref()
        .filter(|langs| !langs.is_empty())
        .map(|langs| langs.iter().map(|cow| cow.to_string()).collect());

    match query!(
        "INSERT INTO posts (did, rkey, blob_cid, blob_mime_type, title, blob_alt_text, \
         tags, languages, created_at, ingested_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, extract(epoch from now())::BIGINT) \
         ON CONFLICT(did, rkey) DO UPDATE SET \
         title = excluded.title, \
         blob_alt_text = excluded.blob_alt_text, \
         tags = excluded.tags, \
         edited_at = extract(epoch from now())::BIGINT",
        record_data.did.as_str(),
        record_data.rkey.as_str(),
        data.gif.blob.blob().cid().as_str(),
        data.gif.blob.blob().mime_type.as_str(),
        data.title.as_str(),
        data.gif.alt.as_ref().map(|v| v.as_str()),
        tags_array.as_deref(),
        languages_array.as_deref(),
        DateTime::parse_from_rfc3339(data.created_at.as_str())
            .unwrap()
            .timestamp()
    )
    .execute(state.database.executor())
    .await
    {
        Ok(_) => {
            info!("Upserted post into database");
            Ok(())
        }
        Err(err) => {
            error!("Failed to upsert post into database: {err:?}");
            Err(err.into())
        }
    }
}

pub async fn handle_post_delete(state: &AppState, record_data: &RecordEventData<'_>) -> Result<()> {
    match query!(
        "DELETE FROM posts WHERE did = $1 AND rkey = $2",
        record_data.did.as_str(),
        record_data.rkey.as_str()
    )
    .execute(state.database.executor())
    .await
    {
        Ok(_) => {
            info!("Deleted post from database");
            Ok(())
        }
        Err(err) => {
            error!("Failed to delete post from database: {err:?}");
            Err(err.into())
        }
    }
}
