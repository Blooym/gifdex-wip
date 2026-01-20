use anyhow::Result;
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
            let cid = Cid::str(cid_str);
            if !cid.is_valid() {
                warn!("Rejected record: invalid CID in rkey");
                return Ok(());
            }
            // Validate rkey CID matches blob CID
            if cid != *data.media.blob.blob().cid() {
                warn!("Rejected record: rkey CID doesn't match blob CID");
                return Ok(());
            }
        }
        None => {
            warn!("Rejected record: rkey doesn't match tid:cid format");
            return Ok(());
        }
    };

    // Loosely-validate the provided blob's mimetype + size.
    if !matches!(
        data.media.blob.blob().mime_type.as_str(),
        "image/gif" | "image/webp"
    ) {
        warn!("Rejected record: blob isn't a valid mimetype");
        return Ok(());
    }
    if data.media.blob.blob().size == 10 * 1024 * 1024 {
        warn!("Rejected record: blob is above maximum size");
        return Ok(());
    }

    // Extract tag/lang data.
    let tags_array = (!data.tags.is_empty()).then(|| {
        data.tags
            .iter()
            .map(|cow| cow.to_string())
            .collect::<Vec<String>>()
    });
    let languages_array = data
        .languages
        .as_ref()
        .filter(|langs| !langs.is_empty())
        .map(|langs| {
            langs
                .iter()
                .map(|cow| cow.to_string())
                .collect::<Vec<String>>()
        });

    match query!(
        "INSERT INTO posts (did, rkey, media_blob_cid, media_blob_mime, title, \
         media_blob_alt, tags, languages, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
         ON CONFLICT(did, rkey) DO UPDATE SET \
         title = excluded.title, \
         media_blob_alt = excluded.media_blob_alt, \
         tags = excluded.tags, \
         edited_at = extract(epoch from now())::BIGINT",
        record_data.did.as_str(),
        record_data.rkey.as_str(),
        data.media.blob.blob().cid().as_str(),
        data.media.blob.blob().mime_type.as_str(),
        data.title.as_str(),
        data.media.alt.as_ref().map(|v| v.as_str()),
        tags_array.as_deref(),
        languages_array.as_deref(),
        data.created_at.as_ref().timestamp_millis()
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
