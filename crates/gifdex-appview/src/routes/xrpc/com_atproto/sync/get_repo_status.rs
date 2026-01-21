use crate::AppState;
use axum::{Json, extract::State};
use jacquard_api::com_atproto::sync::get_repo_status::{
    GetRepoStatusError, GetRepoStatusOutput, GetRepoStatusRequest,
};
use jacquard_axum::{ExtractXrpc, XrpcErrorResponse};
use jacquard_common::xrpc::XrpcError;
use sqlx::query;

pub async fn handle_get_repo_status(
    State(state): State<AppState>,
    ExtractXrpc(request): ExtractXrpc<GetRepoStatusRequest>,
) -> Result<Json<GetRepoStatusOutput<'static>>, XrpcErrorResponse<GetRepoStatusError<'static>>> {
    let record = query!(
        "SELECT is_active, status, rev FROM accounts WHERE did = $1",
        request.did.as_str()
    )
    .fetch_optional(state.database.executor())
    .await
    .unwrap(); // TODO: Use Xrpc error.

    let Some(account) = record else {
        return Err(XrpcError::Xrpc(GetRepoStatusError::RepoNotFound(None)).into());
    };

    Ok(Json(GetRepoStatusOutput {
        active: account.is_active,
        did: request.did,
        rev: account.rev.map(|rev| rev.into()),
        status: Some(account.status.into()),
        extra_data: None,
    }))
}
