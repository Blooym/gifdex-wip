use crate::AppState;
use axum::{Json, extract::State};
use gifdex_lexicons::net_gifdex::actor::{
    ProfileView,
    get_profile::{GetProfileError, GetProfileOutput, GetProfileRequest},
};
use jacquard_axum::{ExtractXrpc, XrpcErrorResponse, service_auth::ExtractOptionalServiceAuth};
use jacquard_common::{
    types::{string::Handle, uri::Uri},
    xrpc::XrpcError,
};
use sqlx::query;

pub async fn handle_get_profile(
    State(state): State<AppState>,
    ExtractOptionalServiceAuth(_auth): ExtractOptionalServiceAuth,
    ExtractXrpc(request): ExtractXrpc<GetProfileRequest>,
) -> Result<Json<GetProfileOutput<'static>>, XrpcErrorResponse<GetProfileError<'static>>> {
    let account = query!(
        "SELECT did, handle, display_name, avatar_blob_cid, pronouns, indexed_at,
        (SELECT COUNT(*) FROM posts WHERE did = accounts.did) as \"post_count!\"
        FROM accounts WHERE did = $1 OR handle = $1",
        request.actor.as_str()
    )
    .fetch_optional(state.database.executor())
    .await
    .unwrap(); // TODO: Use Xrpc error.

    let Some(account) = account else {
        return Err(XrpcError::Xrpc(GetProfileError::ProfileNotFound(None)).into());
    };

    Ok(Json(GetProfileOutput {
        value: ProfileView::new()
            .did(request.actor)
            .handle(
                account
                    .handle
                    .map(|handle| handle.parse::<Handle>().unwrap()),
            )
            .display_name(account.display_name.map(|display_name| display_name.into()))
            .pronouns(account.pronouns.map(|pronouns| pronouns.into()))
            .avatar(account.avatar_blob_cid.map(|blob_cid| {
                Uri::new_owned(
                    state
                        .cdn_url
                        .join(&format!("/avatar/{}/{}", account.did, blob_cid))
                        .unwrap(),
                )
                .unwrap()
            }))
            .post_count(account.post_count)
            .build(),
        extra_data: None,
    }))
}
