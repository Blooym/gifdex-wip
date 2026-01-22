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
use jacquard_axum::{ExtractXrpc, XrpcErrorResponse, service_auth::ExtractOptionalServiceAuth};
use jacquard_common::{
    IntoStatic,
    chrono::{TimeZone, Utc},
    types::{aturi::AtUri, collection::Collection, string::Handle, tid::Tid, uri::Uri},
    xrpc::XrpcError,
};
use sqlx::query;

pub async fn handle_get_posts_by_actor(
    State(state): State<AppState>,
    ExtractOptionalServiceAuth(auth): ExtractOptionalServiceAuth,
    ExtractXrpc(request): ExtractXrpc<GetPostsByActorRequest>,
) -> Result<Json<GetPostsByActorOutput<'static>>, XrpcErrorResponse<GetPostsByActorError<'static>>>
{
    let viewer_did = auth.as_ref().map(|a| a.did().as_str());
    let limit = request.limit.unwrap_or(50).min(100) as i64;
    let posts = query!(
        "SELECT \
            a.did, a.display_name, a.handle, a.avatar_blob_cid, a.indexed_at as account_indexed_at, \
            p.rkey, p.title, p.tags, p.languages, p.media_blob_cid, p.media_blob_mime, \
            p.media_blob_alt, p.media_blob_width, p.media_blob_height, p.created_at, \
            p.edited_at, p.indexed_at as post_indexed_at, \
            (SELECT COUNT(*) FROM post_favourites \
             WHERE post_did = p.did AND post_rkey = p.rkey) as \"favourite_count!\", \
            (SELECT pf.rkey \
             FROM post_favourites pf \
             WHERE pf.post_did = p.did AND pf.post_rkey = p.rkey AND pf.did = $4 \
             LIMIT 1) as \"favourite_rkey\" \
         FROM accounts a \
         INNER JOIN posts p ON a.did = p.did \
         WHERE a.did = $1 AND ($2::BIGINT IS NULL OR p.created_at < $2) \
         ORDER BY p.created_at DESC LIMIT $3",
        request.actor.as_str(),
        request.cursor,
        limit,
        viewer_did
    )
    .fetch_all(state.database.executor())
    .await
    .unwrap(); // TODO: Use Xrpc error

    // If no posts found, check if the account exists.
    if posts.is_empty() {
        let account_exists = query!(
            "SELECT 1 as exists FROM accounts WHERE did = $1",
            request.actor.as_str()
        )
        .fetch_optional(state.database.executor())
        .await
        .unwrap();
        if account_exists.is_none() {
            return Err(XrpcError::Xrpc(GetPostsByActorError::ActorNotFound(None)).into());
        }
    }

    // Generate cursor if we have more posts.
    let cursor = if posts.len() == limit as usize {
        posts.last().map(|post| post.created_at)
    } else {
        None
    };

    // Build post views (if we have any posts)
    let post_views: Vec<PostFeedView> = posts
        .into_iter()
        .map(|post| {
            // Build the profile view from the joined account data
            let profile = ProfileViewBasic::new()
                .did(request.actor.clone())
                .handle(
                    post.handle
                        .clone()
                        .map(|handle| Handle::new_owned(handle).unwrap()),
                )
                .display_name(post.display_name.clone().map(|s| s.into()))
                .avatar(post.avatar_blob_cid.clone().map(|blob_cid| {
                    Uri::new_owned(
                        state
                            .cdn_url
                            .join(&format!("/avatar/{}/{}", post.did, blob_cid))
                            .unwrap(),
                    )
                    .unwrap()
                }))
                .build();

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
                                    .join(&format!("/media/{}/{}", post.did, post.rkey))
                                    .unwrap(),
                            )
                            .unwrap(),
                        )
                        .thumbnail_url(
                            Uri::new_owned(
                                state
                                    .cdn_url
                                    .join(&format!("/media/{}/{}", post.did, post.rkey))
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
                .author(profile)
                .viewer(feed::ViewerState {
                    favourite: post
                        .favourite_rkey
                        .as_ref()
                        .map(|uri| Tid::new(uri.clone()).unwrap()),
                    ..Default::default()
                })
                .created_at(
                    Utc.timestamp_millis_opt(post.created_at)
                        .unwrap()
                        .fixed_offset(),
                )
                .indexed_at(
                    Utc.timestamp_millis_opt(post.post_indexed_at)
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
