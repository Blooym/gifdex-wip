use crate::{AppState, cdn::CdnMediaType};
use axum::{Json, extract::State};
use gifdex_lexicons::net_gifdex::{
    actor::ProfileViewBasic,
    feed::{
        self, PostView, PostViewMedia, PostViewMediaDimensions,
        get_post::{GetPostError, GetPostOutput, GetPostRequest},
        post::Post,
    },
};
use jacquard_axum::{ExtractXrpc, XrpcErrorResponse, service_auth::ExtractOptionalServiceAuth};
use jacquard_common::{
    IntoStatic,
    chrono::{TimeZone, Utc},
    types::{aturi::AtUri, collection::Collection, string::Rkey, uri::Uri},
    xrpc::XrpcError,
};
use sqlx::query;

pub async fn handle_get_post(
    State(state): State<AppState>,
    ExtractOptionalServiceAuth(auth): ExtractOptionalServiceAuth,
    ExtractXrpc(request): ExtractXrpc<GetPostRequest>,
) -> Result<Json<GetPostOutput<'static>>, XrpcErrorResponse<GetPostError<'static>>> {
    let auth_did = auth.as_ref().map(|a| a.did().as_str());
    tracing::debug!("Authenticated DID for request: {auth_did:?}");

    let record = query!(
        r#"SELECT 
          a.did, a.display_name, a.handle, a.avatar_blob_cid, a.indexed_at as account_indexed_at, 
          p.rkey, p.title, p.tags, p.media_blob_cid, p.media_blob_mime, 
          p.media_blob_alt, p.created_at, p.edited_at, p.indexed_at as post_indexed_at, 
          p.media_blob_width, p.media_blob_height,
          (SELECT COUNT(*) FROM post_favourites
             WHERE post_did = p.did AND post_rkey = p.rkey) as "favourite_count!",
           (SELECT pf.rkey FROM post_favourites pf
             WHERE pf.post_did = p.did AND pf.post_rkey = p.rkey AND pf.did = $3
           LIMIT 1) as "favourite_rkey"
         FROM accounts a
         INNER JOIN posts p ON a.did = p.did
         WHERE a.did = $1 AND p.rkey = $2"#,
        request.actor.as_str(),
        request.rkey.as_str(),
        auth_did
    )
    .fetch_optional(state.database.executor())
    .await
    .unwrap(); // TODO: Use Xrpc error.

    let Some(record) = record else {
        return Err(XrpcError::Xrpc(GetPostError::PostNotFound(None)).into());
    };

    // Build profile view.
    let profile_view = ProfileViewBasic::new()
        .did(request.actor.clone())
        .handle(record.handle.map(|h| h.parse().unwrap()))
        .display_name(record.display_name.map(|s| s.into()))
        .avatar(record.avatar_blob_cid.and_then(|bc| {
            Uri::new_owned(state.cdn.make_cdn_url(CdnMediaType::Avatar {
                did: &request.actor,
                cid: &bc.parse().ok()?,
            }))
            .ok()
        }))
        .build();

    // Build post view.
    let post_at_uri = AtUri::from_parts_owned(&record.did, Post::NSID, &record.rkey).unwrap();
    let rkey = Rkey::new(&record.rkey).unwrap();
    let post_view = PostView::new()
        .uri(post_at_uri)
        .title(record.title.into_static())
        .tags(
            record
                .tags
                .map(|tags| tags.into_iter().map(|t| t.into()).collect()),
        )
        .media(
            PostViewMedia::new()
                .fullsize_url(
                    Uri::new_owned(state.cdn.make_cdn_url(CdnMediaType::PostMedia {
                        did: &request.actor,
                        rkey: &rkey,
                        thumbnail: false,
                    }))
                    .unwrap(),
                )
                .thumbnail_url(
                    Uri::new_owned(state.cdn.make_cdn_url(CdnMediaType::PostMedia {
                        did: &request.actor,
                        rkey: &rkey,
                        thumbnail: true,
                    }))
                    .unwrap(),
                )
                .mime_type(record.media_blob_mime.into_static())
                .alt(record.media_blob_alt.map(|s| s.into()))
                .dimensions(
                    PostViewMediaDimensions::new()
                        .height(record.media_blob_height)
                        .width(record.media_blob_width)
                        .build(),
                )
                .build(),
        )
        .favourite_count(record.favourite_count)
        .author(profile_view)
        .viewer(feed::ViewerState {
            favourite: record
                .favourite_rkey
                .as_ref()
                .map(|uri| uri.parse().unwrap()),
            ..Default::default()
        })
        .created_at(
            Utc.timestamp_millis_opt(record.created_at)
                .unwrap()
                .fixed_offset(),
        )
        .indexed_at(
            Utc.timestamp_millis_opt(record.post_indexed_at)
                .unwrap()
                .fixed_offset(),
        )
        .build();

    Ok(Json(GetPostOutput {
        post: post_view,
        extra_data: None,
    }))
}
