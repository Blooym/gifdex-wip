use crate::AppState;
use axum::{Json, extract::State};
use gifdex_lexicons::net_gifdex::moderation::create_report::{
    CreateReportError, CreateReportOutput, CreateReportRequest,
};
use jacquard_axum::{ExtractXrpc, XrpcErrorResponse, service_auth::ExtractServiceAuth};

pub async fn handle_create_report(
    State(_state): State<AppState>,
    ExtractServiceAuth(_auth): ExtractServiceAuth,
    ExtractXrpc(_req): ExtractXrpc<CreateReportRequest>,
) -> Result<Json<CreateReportOutput<'static>>, XrpcErrorResponse<CreateReportError<'static>>> {
    unimplemented!() // TODO: Stub
}
