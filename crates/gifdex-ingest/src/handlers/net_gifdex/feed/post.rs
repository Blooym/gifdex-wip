use crate::AppState;
use anyhow::{Context, Result, bail};
use doubletap::types::RecordEventData;
use gifdex_lexicons::net_gifdex;
use jacquard_common::types::{cid::Cid, tid::Tid};
use sqlx::{PgTransaction, query};
use std::time::Duration;
use tracing::{error, info, warn};
use url::Url;

pub async fn handle_post_create(
    record_data: &RecordEventData<'_>,
    data: &net_gifdex::feed::post::Post<'_>,
    tx: &mut PgTransaction<'_>,
    state: &AppState,
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

    // Extract tag data.
    let tags_array = data
        .tags
        .as_ref()
        .filter(|tags| !tags.is_empty())
        .map(|tags| {
            tags.iter()
                .map(|cow| cow.to_string())
                .collect::<Vec<String>>()
        });

    // TODO: This fetches the blob and does validations every update,
    // but blobs can never change. We need to branch here to do different operations
    // on different calls.
    let pds = state
        .tap_client
        .resolve_did(&record_data.did)
        .await?
        .pds_endpoint()
        .unwrap();
    let response = validate_gif_or_webp(
        &pds.join(&format!(
            "/xrpc/com.atproto.sync.getBlob?did={}&cid={}",
            record_data.did,
            record_data.rkey.split_once(":").unwrap().1
        ))?,
        &state.http_client,
    )
    .await?;

    match query!(
        "INSERT INTO posts (did, rkey, title, media_blob_cid, media_blob_mime, \
         media_blob_alt, media_blob_width, media_blob_height, tags, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
         ON CONFLICT(did, rkey) DO UPDATE SET \
         title = excluded.title, \
         media_blob_alt = excluded.media_blob_alt, \
         tags = excluded.tags, \
         created_at = excluded.created_at, \
         edited_at = extract(epoch from now())::BIGINT",
        record_data.did.as_str(),
        record_data.rkey.as_str(),
        data.title.as_str(),
        data.media.blob.blob().cid().as_str(),
        response.mime_type,
        data.media.alt.as_ref().map(|v| v.as_str()),
        response.width as i64,
        response.height as i64,
        tags_array.as_deref(),
        data.created_at.as_ref().timestamp_millis()
    )
    .execute(&mut **tx)
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

pub async fn handle_post_delete(
    record_data: &RecordEventData<'_>,
    tx: &mut PgTransaction<'_>,
    _state: &AppState,
) -> Result<()> {
    match query!(
        "DELETE FROM posts WHERE did = $1 AND rkey = $2",
        record_data.did.as_str(),
        record_data.rkey.as_str()
    )
    .execute(&mut **tx)
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

async fn validate_gif_or_webp(url: &Url, http_client: &reqwest::Client) -> Result<ImageInfo> {
    let mut buffer = Vec::new();
    let mut response = http_client
        .get(url.as_str())
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .context("Failed to fetch image")?;

    while let Some(chunk) = response.chunk().await.context("Failed to read chunk")? {
        buffer.extend_from_slice(&chunk);

        // Try detection once we have enough bytes
        if buffer.len() >= 32 {
            // Validate MIME type first (fast check)
            if let Some(kind) = infer::get(&buffer) {
                let mime = kind.mime_type();
                if mime != "image/gif" && mime != "image/webp" {
                    bail!("Unsupported format: {}", mime);
                }

                // Then get dimensions
                if let Ok(size) = imagesize::blob_size(&buffer) {
                    // Validate dimensions
                    if size.width > 10_000 || size.height > 10_000 {
                        bail!("Dimensions too large: {}x{}", size.width, size.height);
                    }

                    if size.width == 0 || size.height == 0 {
                        bail!("Invalid dimensions: {}x{}", size.width, size.height);
                    }

                    return Ok(ImageInfo {
                        width: size.width,
                        height: size.height,
                        mime_type: mime.to_string(),
                    });
                }
            }
        }

        if buffer.len() > 32_768 {
            break;
        }
    }

    bail!("Failed to detect image format and dimensions")
}

#[derive(Debug)]
struct ImageInfo {
    width: usize,
    height: usize,
    mime_type: String,
}
