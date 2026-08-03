use chrono::{DateTime, Local, TimeDelta};
use serde_json::Value;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::info;

pub fn filter_identities_by_status(identities: Vec<Value>) -> Vec<Value> {
    info!("Filtering {} identities by Status...", identities.len());
    let filtered_identities: Vec<Value> = identities
        .iter()
        .filter(|identity| identity.get("status").unwrap_or_default().eq("Inactive"))
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

pub async fn dump_identities(identities: &Vec<Value>) -> anyhow::Result<()> {
    let filename = format!("genetec_ident_remover{}.json", Local::now().timestamp());
    info!(
        "Dumping {} relevant identities to file {}",
        identities.len(),
        filename
    );
    let mut file = File::create(filename)
        .await
        .expect("Could not create file to dump identities");
    file.write_all(serde_json::to_string(&identities).unwrap().as_bytes())
        .await?;
    info!("Dump complete");
    Ok(())
}
