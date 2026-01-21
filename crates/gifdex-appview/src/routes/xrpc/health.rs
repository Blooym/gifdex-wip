use crate::AppState;
use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use sqlx::query;

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseStatus {
    Healthy,
    Unhealthy,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HandleHealthResponse {
    status: HealthStatus,
    database: DatabaseStatus,
}

pub async fn handle_health(
    State(state): State<AppState>,
) -> (StatusCode, Json<HandleHealthResponse>) {
    let database_status = match query!("SELECT 1 AS health_check")
        .fetch_one(state.database.executor())
        .await
    {
        Ok(_) => DatabaseStatus::Healthy,
        Err(_) => DatabaseStatus::Unhealthy,
    };

    let status = if database_status == DatabaseStatus::Unhealthy {
        HealthStatus::Unhealthy
    } else {
        HealthStatus::Healthy
    };

    let status_code = match status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    (
        status_code,
        Json(HandleHealthResponse {
            status: status,
            database: database_status,
        }),
    )
}
