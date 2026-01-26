use crate::{AppState, cdn::CdnMediaType};
use axum::{Json, extract::State};
use gifdex_lexicons::net_gifdex::actor::{
    ProfileView,
    get_profiles::{GetProfilesOutput, GetProfilesRequest},
};
use jacquard_axum::{ExtractXrpc, XrpcErrorResponse, service_auth::ExtractOptionalServiceAuth};
use jacquard_common::{
    types::{did::Did, uri::Uri},
    xrpc::GenericXrpcError,
};
use sqlx::query;

pub async fn handle_get_profiles(
    State(state): State<AppState>,
    ExtractOptionalServiceAuth(auth): ExtractOptionalServiceAuth,
    ExtractXrpc(request): ExtractXrpc<GetProfilesRequest>,
) -> Result<Json<GetProfilesOutput<'static>>, XrpcErrorResponse<GenericXrpcError>> {
    let auth_did = auth.as_ref().map(|a| a.did().as_str());
    tracing::debug!("Authenticated DID for request: {auth_did:?}");

    let actors: Vec<String> = request.actors.iter().map(|d| d.to_string()).collect();
    let account = query!(
        r#"SELECT did, handle, display_name, avatar_blob_cid, pronouns, indexed_at,
         (SELECT COUNT(*) FROM posts WHERE did = accounts.did) as "post_count!"
         FROM accounts WHERE did = ANY($1)"#,
        &actors
    )
    .fetch_all(state.database.executor())
    .await
    .unwrap(); // TODO: Use Xrpc error.

    Ok(Json(GetProfilesOutput {
        profiles: account
            .into_iter()
            .map(|account| {
                let did: Did = account.did.parse().unwrap();
                ProfileView::new()
                    .did(did.clone())
                    .handle(account.handle.map(|handle| handle.parse().unwrap()))
                    .display_name(account.display_name.map(|s| s.into()))
                    .pronouns(account.pronouns.map(|pronouns| pronouns.into()))
                    .avatar(account.avatar_blob_cid.and_then(|bc| {
                        Uri::new_owned(state.cdn.make_cdn_url(CdnMediaType::Avatar {
                            did: &did,
                            cid: &bc.parse().ok()?,
                        }))
                        .ok()
                    }))
                    .post_count(account.post_count)
                    .build()
            })
            .collect(),
        extra_data: None,
    }))
}
