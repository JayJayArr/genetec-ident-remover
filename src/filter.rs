use chrono::{DateTime, TimeDelta};
use serde_json::Value;
use tracing::info;

pub fn filter_identities_by_status(identities: Vec<Value>) -> Vec<Value> {
    info!("Filtering {} identities by Status...", identities.len());
    let filtered_identities: Vec<Value> = identities
        .iter()
        .filter(|identity| {
            identity
                .get("status")
                .unwrap_or_default()
                .to_string()
                .contains("Inactive")
        })
        .cloned()
        .collect();

    info!(
        "{} identities remaining after filtering by Status.",
        filtered_identities.len()
    );
    filtered_identities
}

pub fn filter_identities_by_lastmodified(identities: Vec<Value>, inactive_days: i64) -> Vec<Value> {
    info!(
        "Filtering {} identities by lastModificationDateUtc...",
        identities.len()
    );

    let now = chrono::Utc::now();

    let identities: Vec<Value> = identities
        .iter()
        .filter(|identity| {
            let lastmodified = identity
                .get("lastModificationDateUtc")
                .unwrap_or_default()
                .as_str()
                .unwrap();
            let lastmodifier_datetime =
                DateTime::parse_from_rfc3339(lastmodified).unwrap().to_utc();
            let timediff = now - lastmodifier_datetime;
            timediff > TimeDelta::days(inactive_days)
        })
        .cloned()
        .collect();

    info!(
        "{} identities remaining after filtering by lastModificationDateUtc.",
        identities.len()
    );
    identities
}
