pub mod actor;
pub mod feed;
pub mod identity;
pub mod moderation;

use crate::AppState;
use anyhow::{Result, bail};
use floodgate::api::RecordEventData;
use tracing::error;

pub async fn handle_unknown_event(
    _state: &AppState,
    record_data: &RecordEventData<'_>,
) -> Result<()> {
    error!(
        "No handler for record data: {record_data:?}\nIf tap is configured correctly then this message is considered a bug."
    );
    bail!("No registered handle for event type");
}
