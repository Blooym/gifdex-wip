use crate::AppState;
use axum::{Json, extract::State};
use gifdex_lexicons::net_gifdex::{
    actor::ProfileViewBasic,
    feed::{
        self, PostView, PostViewMedia, PostViewMediaDimensions,
        get_post::{GetPostError, GetPostOutput, GetPostRequest},
    },
};
use jacquard_axum::{ExtractXrpc, XrpcErrorResponse, service_auth::ExtractOptionalServiceAuth};
use jacquard_common::{
    IntoStatic,
    chrono::{TimeZone, Utc},
    types::{aturi::AtUri, string::Handle, tid::Tid, uri::Uri},
    xrpc::XrpcError,
};
use sqlx::query;

pub async fn handle_get_post(
    State(state): State<AppState>,
    ExtractOptionalServiceAuth(auth): ExtractOptionalServiceAuth,
    ExtractXrpc(request): ExtractXrpc<GetPostRequest>,
) -> Result<Json<GetPostOutput<'static>>, XrpcErrorResponse<GetPostError<'static>>> {
    let viewer_did = auth.as_ref().map(|a| a.did().as_str());
    let result = query!(
        "SELECT  \
            a.did, a.display_name, a.handle, a.avatar_blob_cid, a.indexed_at as account_indexed_at, \
            p.rkey, p.title, p.tags, p.languages, p.media_blob_cid, p.media_blob_mime, \
            p.media_blob_alt, p.created_at, p.edited_at, p.indexed_at as post_indexed_at, \
            p.media_blob_width, p.media_blob_height, \
            (SELECT COUNT(*) FROM post_favourites \
            WHERE post_did = p.did AND post_rkey = p.rkey) as \"favourite_count!\", \
            (SELECT pf.rkey \
            FROM post_favourites pf \
            WHERE pf.post_did = p.did AND pf.post_rkey = p.rkey AND pf.did = $3 \
            LIMIT 1) as \"favourite_rkey\" \
         FROM accounts a \
         INNER JOIN posts p ON a.did = p.did \
         WHERE a.did = $1 AND p.rkey = $2",
        request.actor.as_str(),
        request.rkey.as_str(),
        viewer_did
    )
    .fetch_optional(state.database.executor())
    .await
    .unwrap(); // TODO: Use Xrpc error.

    let Some(result) = result else {
        return Err(XrpcError::Xrpc(GetPostError::PostNotFound(None)).into());
    };

    // Build post view.
    let uri = AtUri::new_owned(format!(
        "at://{}/net.gifdex.feed.post/{}",
        result.did, result.rkey
    ))
    .unwrap();
    let post_view = PostView::new()
        .uri(uri)
        .title(result.title.into_static())
        .tags(
            result
                .tags
                .map(|tags| tags.into_iter().map(|t| t.into()).collect()),
        )
        .languages(
            result
                .languages
                .map(|langs| langs.into_iter().map(|l| l.into()).collect()),
        )
        .media(
            PostViewMedia::new()
                .fullsize_url(
                    Uri::new_owned(
                        state
                            .cdn_url
                            .join(&format!("/media/{}/{}", result.did, result.rkey))
                            .unwrap(),
                    )
                    .unwrap(),
                )
                .thumbnail_url(
                    Uri::new_owned(
                        state
                            .cdn_url
                            .join(&format!("/media/{}/{}", result.did, result.rkey))
                            .unwrap(),
                    )
                    .unwrap(),
                )
                .mime_type(result.media_blob_mime.into_static())
                .alt(result.media_blob_alt.map(|s| s.into()))
                .dimensions(
                    PostViewMediaDimensions::new()
                        .height(result.media_blob_height)
                        .width(result.media_blob_width)
                        .build(),
                )
                .build(),
        )
        .favourite_count(result.favourite_count)
        .author(
            ProfileViewBasic::new()
                .did(request.actor.clone())
                .handle(
                    result
                        .handle
                        .map(|handle| handle.parse::<Handle>().unwrap()),
                )
                .display_name(result.display_name.map(|s| s.into()))
                .avatar(result.avatar_blob_cid.map(|blob_cid| {
                    Uri::new_owned(
                        state
                            .cdn_url
                            .join(&format!("/avatar/{}/{}", result.did, blob_cid))
                            .unwrap(),
                    )
                    .unwrap()
                }))
                .build(),
        )
        .viewer(feed::ViewerState {
            favourite: result
                .favourite_rkey
                .as_ref()
                .map(|uri| uri.parse::<Tid>().unwrap()),
            ..Default::default()
        })
        .created_at(
            Utc.timestamp_millis_opt(result.created_at)
                .unwrap()
                .fixed_offset(),
        )
        .indexed_at(
            Utc.timestamp_millis_opt(result.post_indexed_at)
                .unwrap()
                .fixed_offset(),
        )
        .build();
    Ok(Json(GetPostOutput {
        post: post_view,
        extra_data: None,
    }))
}
