use crate::{AppState, cdn::CdnMediaType};
use axum::{Json, extract::State};
use gifdex_lexicons::net_gifdex::{
    actor::ProfileViewBasic,
    feed::{
        self, PostFeedView, PostViewMedia, PostViewMediaDimensions,
        get_posts_by_actor::{
            GetPostsByActorError, GetPostsByActorOutput, GetPostsByActorRequest,
            GetPostsByActorSortBy,
        },
        post::Post,
    },
};
use jacquard_axum::{ExtractXrpc, XrpcErrorResponse, service_auth::ExtractOptionalServiceAuth};
use jacquard_common::{
    IntoStatic,
    chrono::{TimeZone, Utc},
    types::{
        aturi::AtUri,
        collection::Collection,
        string::{Handle, Rkey},
        tid::Tid,
        uri::Uri,
    },
    xrpc::XrpcError,
};
use sqlx::query;

pub async fn handle_get_posts_by_actor(
    State(state): State<AppState>,
    ExtractOptionalServiceAuth(auth): ExtractOptionalServiceAuth,
    ExtractXrpc(request): ExtractXrpc<GetPostsByActorRequest>,
) -> Result<Json<GetPostsByActorOutput<'static>>, XrpcErrorResponse<GetPostsByActorError<'static>>>
{
    let auth_did = auth.as_ref().map(|a| a.did().as_str());
    tracing::debug!("Authenticated DID for request: {auth_did:?}");

    let limit = request.limit.unwrap_or(50).min(100);
    let sort_by = request.sort_by.unwrap_or(GetPostsByActorSortBy::Newest);

    // Execute the appropriate compile-time checked query based on sort mode
    //
    // Opinions? Me too.
    // https://tenor.com/view/i-have-gone-too-far-scared-shocked-gif-22861850
    struct DatabasePostData {
        did: String,
        display_name: Option<String>,
        handle: Option<String>,
        avatar_blob_cid: Option<String>,
        rkey: String,
        title: String,
        tags: Option<Vec<String>>,
        media_blob_mime: String,
        media_blob_alt: Option<String>,
        media_blob_width: i32,
        media_blob_height: i32,
        created_at: i64,
        edited_at: Option<i64>,
        post_indexed_at: i64,
        favourite_count: i64,
        favourite_rkey: Option<String>,
    }
    let posts: Vec<DatabasePostData> = match sort_by {
        GetPostsByActorSortBy::Oldest => {
            let results = query!(
                r#"SELECT
                  a.did, a.display_name, a.handle, a.avatar_blob_cid, a.indexed_at as account_indexed_at,
                  p.rkey, p.title, p.tags, p.media_blob_mime,
                  p.media_blob_alt, p.media_blob_width, p.media_blob_height, p.created_at,
                  p.edited_at, p.indexed_at as post_indexed_at,
                  (SELECT COUNT(*) FROM post_favourites
                     WHERE post_did = p.did AND post_rkey = p.rkey) as "favourite_count!",
                  (SELECT pf.rkey FROM post_favourites pf
                   WHERE pf.post_did = p.did AND pf.post_rkey = p.rkey AND pf.did = $4
                   LIMIT 1) as "favourite_rkey"
                 FROM accounts a
                 INNER JOIN posts p ON a.did = p.did
                 WHERE a.did = $1 AND ($2::BIGINT IS NULL OR p.created_at > $2)
                 ORDER BY p.created_at ASC LIMIT $3"#,
                request.actor.as_str(),
                request.cursor,
                limit,
                auth_did
            )
            .fetch_all(state.database.executor())
            .await
            .unwrap(); // TODO: Use Xrpc error

            results
                .into_iter()
                .map(|r| DatabasePostData {
                    did: r.did,
                    display_name: r.display_name,
                    handle: r.handle,
                    avatar_blob_cid: r.avatar_blob_cid,
                    rkey: r.rkey,
                    title: r.title,
                    tags: r.tags,
                    media_blob_mime: r.media_blob_mime,
                    media_blob_alt: r.media_blob_alt,
                    media_blob_width: r.media_blob_width,
                    media_blob_height: r.media_blob_height,
                    created_at: r.created_at,
                    edited_at: r.edited_at,
                    post_indexed_at: r.post_indexed_at,
                    favourite_count: r.favourite_count,
                    favourite_rkey: r.favourite_rkey,
                })
                .collect()
        }
        GetPostsByActorSortBy::Top => {
            let results = query!(
                r#"SELECT
                  a.did, a.display_name, a.handle, a.avatar_blob_cid, a.indexed_at as account_indexed_at,
                  p.rkey, p.title, p.tags, p.media_blob_mime,
                  p.media_blob_alt, p.media_blob_width, p.media_blob_height, p.created_at,
                  p.edited_at, p.indexed_at as post_indexed_at,
                  (SELECT COUNT(*) FROM post_favourites
                     WHERE post_did = p.did AND post_rkey = p.rkey) as "favourite_count!",
                  (SELECT pf.rkey FROM post_favourites pf
                   WHERE pf.post_did = p.did AND pf.post_rkey = p.rkey AND pf.did = $4
                   LIMIT 1) as "favourite_rkey"
                 FROM accounts a
                 INNER JOIN posts p ON a.did = p.did
                 WHERE a.did = $1 AND ($2::BIGINT IS NULL OR p.created_at < $2)
                 ORDER BY (SELECT COUNT(*) FROM post_favourites WHERE post_did = p.did AND post_rkey = p.rkey) DESC, p.created_at DESC
                 LIMIT $3"#,
                request.actor.as_str(),
                request.cursor,
                limit,
                auth_did
            )
            .fetch_all(state.database.executor())
            .await
            .unwrap(); // TODO: Use Xrpc error

            results
                .into_iter()
                .map(|r| DatabasePostData {
                    did: r.did,
                    display_name: r.display_name,
                    handle: r.handle,
                    avatar_blob_cid: r.avatar_blob_cid,
                    rkey: r.rkey,
                    title: r.title,
                    tags: r.tags,
                    media_blob_mime: r.media_blob_mime,
                    media_blob_alt: r.media_blob_alt,
                    media_blob_width: r.media_blob_width,
                    media_blob_height: r.media_blob_height,
                    created_at: r.created_at,
                    edited_at: r.edited_at,
                    post_indexed_at: r.post_indexed_at,
                    favourite_count: r.favourite_count,
                    favourite_rkey: r.favourite_rkey,
                })
                .collect()
        }
        GetPostsByActorSortBy::Newest => {
            let results = query!(
                r#"SELECT
                  a.did, a.display_name, a.handle, a.avatar_blob_cid, a.indexed_at as account_indexed_at,
                  p.rkey, p.title, p.tags, p.media_blob_mime,
                  p.media_blob_alt, p.media_blob_width, p.media_blob_height, p.created_at,
                  p.edited_at, p.indexed_at as post_indexed_at,
                  (SELECT COUNT(*) FROM post_favourites
                     WHERE post_did = p.did AND post_rkey = p.rkey) as "favourite_count!",
                  (SELECT pf.rkey FROM post_favourites pf
                   WHERE pf.post_did = p.did AND pf.post_rkey = p.rkey AND pf.did = $4
                   LIMIT 1) as "favourite_rkey"
                 FROM accounts a
                 INNER JOIN posts p ON a.did = p.did
                 WHERE a.did = $1 AND ($2::BIGINT IS NULL OR p.created_at < $2)
                 ORDER BY p.created_at DESC LIMIT $3"#,
                request.actor.as_str(),
                request.cursor,
                limit,
                auth_did
            )
            .fetch_all(state.database.executor())
            .await
            .unwrap(); // TODO: Use Xrpc error

            results
                .into_iter()
                .map(|r| DatabasePostData {
                    did: r.did,
                    display_name: r.display_name,
                    handle: r.handle,
                    avatar_blob_cid: r.avatar_blob_cid,
                    rkey: r.rkey,
                    title: r.title,
                    tags: r.tags,
                    media_blob_mime: r.media_blob_mime,
                    media_blob_alt: r.media_blob_alt,
                    media_blob_width: r.media_blob_width,
                    media_blob_height: r.media_blob_height,
                    created_at: r.created_at,
                    edited_at: r.edited_at,
                    post_indexed_at: r.post_indexed_at,
                    favourite_count: r.favourite_count,
                    favourite_rkey: r.favourite_rkey,
                })
                .collect()
        }
    };

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

    // Build ProfileView
    let profile = if let Some(first) = posts.first() {
        Some(
            ProfileViewBasic::new()
                .did(request.actor.clone())
                .handle(
                    first
                        .handle
                        .clone()
                        .and_then(|handle| Handle::new_owned(handle).ok()),
                )
                .display_name(first.display_name.clone().map(|s| s.into()))
                .avatar(first.avatar_blob_cid.clone().and_then(|blob_cid| {
                    Uri::new_owned(state.cdn.make_cdn_url(CdnMediaType::Avatar {
                        did: &request.actor,
                        cid: &blob_cid.parse().ok()?,
                    }))
                    .ok()
                }))
                .build(),
        )
    } else {
        None
    };

    // Build PostFeedViews (if we have any posts)
    let post_views: Vec<PostFeedView> = posts
        .into_iter()
        .map(|post| {
            let post_at_uri = AtUri::from_parts_owned(&post.did, Post::NSID, &post.rkey).unwrap();
            let rkey = Rkey::new(&post.rkey).unwrap();
            PostFeedView::new()
                .uri(post_at_uri)
                .title(post.title.into_static())
                .tags(
                    post.tags
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
                .author(
                    profile
                        .clone()
                        .expect("profile should exist if posts exist"),
                )
                .viewer(feed::ViewerState {
                    favourite: post
                        .favourite_rkey
                        .as_ref()
                        .map(|uri| Tid::new(uri).unwrap()),
                    ..Default::default()
                })
                .created_at(
                    Utc.timestamp_millis_opt(post.created_at)
                        .unwrap()
                        .fixed_offset(),
                )
                .edited_at(
                    post.edited_at
                        .map(|e| Utc.timestamp_millis_opt(e).unwrap().fixed_offset().into()),
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
