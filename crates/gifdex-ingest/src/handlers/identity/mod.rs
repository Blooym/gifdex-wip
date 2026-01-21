use crate::AppState;
use anyhow::Result;
use floodgate::api::IdentityEventData;
use sqlx::{PgTransaction, query};
use tracing::{error, info};

pub async fn handle_identity(
    identity: &IdentityEventData<'_>,
    tx: &mut PgTransaction<'_>,
    state: &AppState,
) -> Result<()> {
    // Completely purge data related to accounts that are deleted or takendown.
    // Note: this does not delete any labels applied to the account or their content.
    if matches!(identity.status.as_str(), "deleted" | "takendown") {
        if let Err(err) = query!("DELETE FROM accounts WHERE did = $1", identity.did.as_str())
            .execute(&mut **tx)
            .await
        {
            error!("Failed to delete account: {err:?}");
            return Err(err.into());
        };
        info!("Removed all userdata for account as it was deleted or takendown");
        return Ok(());
    }

    let pds = state
        .tap_client
        .resolve_did(&identity.did)
        .await?
        .pds_endpoint()
        .map(|pds| pds.host_str().unwrap().to_string());

    // Update state of account incase of handle/status/is_active updates.
    match query!(
        "INSERT INTO accounts (did, handle, pds, is_active, status, created_at) \
         VALUES ($1, $2, $3, $4, $5, (extract(epoch from now()) * 1000)::BIGINT) \
         ON CONFLICT(did) DO UPDATE SET \
         handle = excluded.handle, \
         is_active = excluded.is_active, \
         pds = excluded.pds, \
         status = excluded.status",
        identity.did.as_str(),
        identity.handle.as_str(),
        pds,
        identity.is_active,
        identity.status
    )
    .execute(&mut **tx)
    .await
    {
        Ok(_) => {
            info!("Upserted stored account data into database");
            Ok(())
        }
        Err(err) => {
            error!("Failed to upsert account data into database: {err:?}");
            Err(err.into())
        }
    }
}
