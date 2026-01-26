use crate::AppState;
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
};
use reqwest::{StatusCode, header};
use std::{
    sync::OnceLock,
    time::SystemTime,
};

pub async fn handle_well_known_did(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    // Best attempt at helping clients avoid updating their internal cache when unnecessary.
    // Just send the first-request time as the service document cannot change at runtime.
    static LAST_MODIFIED: OnceLock<(SystemTime, String)> = OnceLock::new();
    let last_modified = LAST_MODIFIED.get_or_init(|| {
        let http = httpdate::fmt_http_date(SystemTime::now());
        let truncated = httpdate::parse_http_date(&http).unwrap();
        (truncated, http)
    });
    if headers
        .get(header::IF_MODIFIED_SINCE)
        .and_then(|value| value.to_str().ok())
        .and_then(|date| httpdate::parse_http_date(date).ok())
        .is_some_and(|since| since >= last_modified.0)
    {
        return StatusCode::NOT_MODIFIED.into_response();
    }

    (
        [(
            axum::http::header::LAST_MODIFIED,
            HeaderValue::from_str(&last_modified.1).unwrap(),
        )],
        Json(&state.service_did_document),
    )
        .into_response()
}
