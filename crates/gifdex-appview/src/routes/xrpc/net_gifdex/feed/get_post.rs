use crate::AppState;
use axum::{Json, extract::State};
use gifdex_lexicons::net_gifdex::{
    actor::ProfileViewBasic,
    feed::{
        self, PostView, PostViewMedia, PostViewMediaDimensions,
        get_post::{GetPostError, GetPostOutput, GetPostRequest},
    },
};
use jacquard_axum::{ExtractXrpc, XrpcErrorResponse};
use jacquard_common::{
    IntoStatic,
    chrono::{TimeZone, Utc},
    types::{aturi::AtUri, string::Handle, uri::Uri},
    xrpc::XrpcError,
};
use sqlx::query;

pub async fn handle_get_post(
    State(state): State<AppState>,
    ExtractXrpc(request): ExtractXrpc<GetPostRequest>,
) -> Result<Json<GetPostOutput<'static>>, XrpcErrorResponse<GetPostError<'static>>> {
    let account = query!(
        "SELECT did, display_name, handle, avatar_blob_cid, indexed_at
        FROM accounts WHERE did = $1",
        request.actor.as_str()
    )
    .fetch_optional(state.database.executor())
    .await
    .unwrap(); // TODO: Use Xrpc error.

    let Some(account) = account else {
        return Err(XrpcError::Xrpc(GetPostError::ActorNotFound(None)).into());
    };

    // Build the profile view
    let profile = ProfileViewBasic::new()
        .did(request.actor.clone())
        .handle(
            account
                .handle
                .map(|handle| Handle::new_owned(handle).unwrap()),
        )
        .display_name(account.display_name.map(|s| s.into()))
        .avatar(account.avatar_blob_cid.map(|blob_cid| {
            Uri::new_owned(
                state
                    .cdn_url
                    .join(&format!("/avatar/{}/{}", account.did, blob_cid))
                    .unwrap(),
            )
            .unwrap()
        }))
        .build();

    // Build the post view.
    let post = query!(
        "SELECT did, rkey, title, tags, languages, media_blob_cid, media_blob_mime, 
                media_blob_alt, created_at, edited_at, indexed_at, media_blob_width, media_blob_height,
                (SELECT COUNT(*) FROM post_favourites WHERE post_did = posts.did AND post_rkey = posts.rkey) as \"favourite_count!\"
         FROM posts WHERE did = $1 AND rkey = $2", request.actor.as_str(), request.rkey.as_str()
    )
    .fetch_optional(state.database.executor())
    .await
    .unwrap(); // TODO: Use Xrpc error

    let Some(post) = post else {
        return Err(XrpcError::Xrpc(GetPostError::PostNotFound(None)).into());
    };

    // Build post views
    let uri = AtUri::new_owned(format!(
        "at://{}/net.gifdex.feed.post/{}",
        post.did, post.rkey
    ))
    .unwrap();
    let post_view = PostView::new()
        .uri(uri)
        .title(post.title.into_static())
        .tags(
            post.tags
                .map(|tags| tags.into_iter().map(|t| t.into()).collect()),
        )
        .languages(
            post.languages
                .map(|langs| langs.into_iter().map(|l| l.into()).collect()),
        )
        .media(
            PostViewMedia::new()
                .fullsize_url(
                    Uri::new_owned(
                        state
                            .cdn_url
                            .join(&format!("/media/{}/{}", account.did, post.rkey))
                            .unwrap(),
                    )
                    .unwrap(),
                )
                .thumbnail_url(
                    Uri::new_owned(
                        state
                            .cdn_url
                            .join(&format!("/media/{}/{}", account.did, post.rkey))
                            .unwrap(),
                    )
                    .unwrap(),
                )
                .mime_type(post.media_blob_mime.into_static())
                .alt(post.media_blob_alt.map(|s| s.into()))
                .dimensions(
                    PostViewMediaDimensions::new()
                        .height(post.media_blob_height)
                        .width(post.media_blob_width)
                        .build(),
                )
                .build(),
        )
        .favourite_count(post.favourite_count)
        .author(profile.clone())
        .viewer(feed::ViewerState::new().favourited(false).build())
        .created_at(
            Utc.timestamp_millis_opt(post.created_at)
                .unwrap()
                .fixed_offset(),
        )
        .indexed_at(
            Utc.timestamp_millis_opt(post.indexed_at)
                .unwrap()
                .fixed_offset(),
        )
        .build();

    Ok(Json(GetPostOutput {
        post: post_view,
        extra_data: None,
    }))
}
