use crate::AppState;
use axum::{Json, extract::State};
use gifdex_lexicons::net_gifdex::{
    actor::ProfileViewBasic,
    feed::{
        self, PostFeedView, PostViewMedia, PostViewMediaDimensions,
        get_posts_by_actor::{GetPostsByActorError, GetPostsByActorOutput, GetPostsByActorRequest},
        post::Post,
    },
};
use jacquard_axum::{ExtractXrpc, XrpcErrorResponse};
use jacquard_common::{
    IntoStatic,
    chrono::{TimeZone, Utc},
    types::{aturi::AtUri, collection::Collection, string::Handle, uri::Uri},
    xrpc::XrpcError,
};
use sqlx::query;

pub async fn handle_get_posts_by_actor(
    State(state): State<AppState>,
    ExtractXrpc(request): ExtractXrpc<GetPostsByActorRequest>,
) -> Result<Json<GetPostsByActorOutput<'static>>, XrpcErrorResponse<GetPostsByActorError<'static>>>
{
    let account = query!(
        "SELECT did, display_name, handle, avatar_blob_cid, indexed_at
        FROM accounts WHERE did = $1",
        request.actor.as_str()
    )
    .fetch_optional(state.database.executor())
    .await
    .unwrap(); // TODO: Use Xrpc error.

    let Some(account) = account else {
        return Err(XrpcError::Xrpc(GetPostsByActorError::ActorNotFound(None)).into());
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

    // Use cursor if provided & fetch posts with pagination.
    // Cursor in our case is the creation date of the record.
    // TODO: Investigate if that's the best case as records can be moved by users.
    let limit = request.limit.unwrap_or(50).min(100) as i64;
    let posts = query!(
        "SELECT did, rkey, title, tags, languages, media_blob_cid, media_blob_mime, 
         media_blob_alt,  media_blob_width, media_blob_height, created_at, \
         edited_at, indexed_at, \
         (SELECT COUNT(*) FROM post_favourites \
         WHERE post_did = posts.did AND post_rkey = posts.rkey) as \"favourite_count!\"
         FROM posts WHERE did = $1 AND ($2::BIGINT IS NULL OR created_at < $2)
         ORDER BY created_at DESC LIMIT $3",
        request.actor.as_str(),
        request.cursor,
        limit
    )
    .fetch_all(state.database.executor())
    .await
    .unwrap(); // TODO: Use Xrpc error

    // Generate cursor if we have more posts.
    let cursor = if posts.len() == limit as usize {
        posts.last().map(|post| post.created_at)
    } else {
        None
    };

    // Build post views.
    let post_views: Vec<PostFeedView> = posts
        .into_iter()
        .map(|post| {
            let uri = AtUri::new_owned(format!("at://{}/{}/{}", post.did, Post::NSID, post.rkey))
                .unwrap();
            PostFeedView::new()
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
                .build()
        })
        .collect();

    Ok(Json(GetPostsByActorOutput {
        feed: post_views,
        cursor,
        extra_data: None,
    }))
}
