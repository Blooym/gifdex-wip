use crate::AppState;
use axum::{Json, extract::State};
use gifdex_lexicons::net_gifdex::feed::get_posts_by_query::{
    GetPostsByQueryError, GetPostsByQueryOutput, GetPostsByQueryRequest,
};
use jacquard_axum::{ExtractXrpc, XrpcErrorResponse, service_auth::ExtractOptionalServiceAuth};

pub async fn handle_get_posts_by_query(
    State(_state): State<AppState>,
    ExtractOptionalServiceAuth(_auth): ExtractOptionalServiceAuth,
    ExtractXrpc(_request): ExtractXrpc<GetPostsByQueryRequest>,
) -> Result<Json<GetPostsByQueryOutput<'static>>, XrpcErrorResponse<GetPostsByQueryError<'static>>>
{
    // TODO: stub
    unimplemented!()
}
