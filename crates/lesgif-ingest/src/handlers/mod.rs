pub mod actor;
pub mod feed;
pub mod identity;
pub mod moderation;

use crate::{AppState, tap::TapRecordEventData};
use tracing::error;

pub async fn handle_unknown_event(_state: &AppState, record_data: &TapRecordEventData<'_>) -> bool {
    error!(
        "No handler for record data: {record_data:?}\nIf tap is configured correctly then this message is considered a bug."
    );
    false
}
