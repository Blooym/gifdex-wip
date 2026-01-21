use crate::AppState;
use axum::{Json, extract::State};
use gifdex_lexicons::net_gifdex::actor::{
    ProfileView,
    get_profiles::{GetProfilesOutput, GetProfilesRequest},
};
use jacquard_axum::{ExtractXrpc, XrpcErrorResponse};
use jacquard_common::{
    types::{did::Did, string::Handle, uri::Uri},
    xrpc::GenericXrpcError,
};
use sqlx::query;

pub async fn handle_get_profiles(
    State(state): State<AppState>,
    ExtractXrpc(request): ExtractXrpc<GetProfilesRequest>,
) -> Result<Json<GetProfilesOutput<'static>>, XrpcErrorResponse<GenericXrpcError>> {
    let actors: Vec<String> = request.actors.iter().map(|d| d.to_string()).collect();
    let account = query!(
        "SELECT did, handle, display_name, avatar_blob_cid, pronouns, indexed_at,
         (SELECT COUNT(*) FROM posts WHERE did = accounts.did) as \"post_count!\"
         FROM accounts WHERE did = ANY($1) OR handle = ANY($1)",
        &actors
    )
    .fetch_all(state.database.executor())
    .await
    .unwrap(); // TODO: Use Xrpc error.

    Ok(Json(GetProfilesOutput {
        profiles: account
            .into_iter()
            .map(|account| {
                ProfileView::new()
                    .did(account.did.parse::<Did>().unwrap())
                    .handle(
                        account
                            .handle
                            .map(|handle| handle.parse::<Handle>().unwrap()),
                    )
                    .display_name(account.display_name.map(|s| s.into()))
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
                    .build()
            })
            .collect(),
        extra_data: None,
    }))
}
