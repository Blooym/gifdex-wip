use crate::AppState;
use anyhow::Result;
use floodgate::api::RecordEventData;
use gifdex_lexicons::net_gifdex;
use sqlx::query;
use tracing::{error, info};

pub async fn handle_rule_create_event(
    state: &AppState,
    record_data: &RecordEventData<'_>,
    data: &net_gifdex::labeler::rule::Rule<'_>,
) -> Result<()> {
    // Determine behaviour type and extract fields based on behaviour variant.
    let (behaviour, default_setting, adult_content, takedown) = match &data.behaviour {
        net_gifdex::labeler::rule::RuleBehaviour::Annotate(annotate) => (
            "annotate",
            Some(annotate.default_setting.as_str()),
            Some(annotate.adult_content),
            None,
        ),
        net_gifdex::labeler::rule::RuleBehaviour::Moderate(moderate) => {
            ("moderate", None, None, Some(moderate.takedown))
        }
        behaviour @ _ => {
            tracing::warn!("Rejected record: unknown rule_behaviour: {behaviour:?}");
            return Ok(());
        }
    };

    match query!(
        r#"INSERT INTO labeler_rules (
            rkey, did, name, description, behaviour,
            default_setting, adult_content, takedown,
            created_at, indexed_at
        ) VALUES (
            $1, $2, $3, $4, $5::TEXT::labeler_behaviour, $6::TEXT::labeler_behaviour_setting, $7, $8,
            $9, extract(epoch from now())::BIGINT
        )
        ON CONFLICT(did, rkey) DO UPDATE SET
            name = excluded.name,
            description = excluded.description,
            behaviour = excluded.behaviour,
            default_setting = excluded.default_setting,
            adult_content = excluded.adult_content,
            takedown = excluded.takedown,
            created_at = excluded.created_at,
            edited_at = extract(epoch from now())::BIGINT"#,
        record_data.rkey.as_str(),
        record_data.did.as_str(),
        data.name.as_str(),
        data.description.as_str(),
        behaviour,
        default_setting,
        adult_content,
        takedown,
        data.created_at.as_ref().timestamp_millis()
    )
    .execute(state.database.executor())
    .await
    {
        Ok(_) => {
            info!("Upserted labeler rule");
            Ok(())
        }
        Err(err) => {
            error!("Failed to upsert labeler rule: {err:?}");
            Err(err.into())
        }
    }
}

pub async fn handle_rule_delete_event(
    state: &AppState,
    record_data: &RecordEventData<'_>,
) -> Result<()> {
    match query!(
        "DELETE FROM labeler_rules WHERE did = $1 AND rkey = $2",
        record_data.did.as_str(),
        record_data.rkey.as_str()
    )
    .execute(state.database.executor())
    .await
    {
        Ok(_) => {
            info!("Deleted labeler rule");
            Ok(())
        }
        Err(err) => {
            error!("Failed to delete labeler rule: {err:?}");
            Err(err.into())
        }
    }
}
