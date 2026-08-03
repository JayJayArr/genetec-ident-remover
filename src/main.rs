use crate::endpoint::{delete_identities, get_all_identities, get_bearer_token};
use crate::filter::{
    dump_identities, filter_identities_by_lastmodified, filter_identities_by_status,
};
use crate::key::KeyFile;
use crate::telemetry::init_tracing;
use clap::Parser;
use oauth2::TokenResponse;
use tracing::info;
use tracing::warn;
mod endpoint;
mod filter;
mod key;
mod telemetry;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Integration key-file from Genetec to authenticate
    #[arg(short)]
    keyfile: String,

    /// Display users, but do not delete
    #[arg(long, default_value_t = true)]
    dry_run: bool,

    /// Minimum Inactivity Period in days for an `Identity` to be deleted
    #[arg(short, long, default_value_t = 90)]
    inactive_days: i64,

    /// Number of concurrent requests when deleting the Identities
    #[arg(short, long, default_value_t = 10)]
    concurrency: usize,
}

#[tokio::main]

async fn main() -> anyhow::Result<()> {
    init_tracing()?;
    let args = Args::parse();
    if !args.dry_run {
        warn!(
            "This runs destructive action, please run with --dry-run before running in non-dry-run mode"
        )
    }

    let file = tokio::fs::read_to_string(args.keyfile).await.unwrap();

    let key_values: KeyFile = serde_json::from_str(file.as_str())?;

    let tokenresponse = get_bearer_token(
        key_values.clientId,
        key_values.clientSecret,
        format!("{}/connect/token", key_values.stsUrl),
    )
    .await?;
    let bearer_token = tokenresponse.access_token().secret();

    let mut identities_response = get_all_identities(
        bearer_token,
        key_values.identityServiceUrl.clone(),
        key_values.accountId.clone(),
    )
    .await?;

    identities_response = filter_identities_by_status(identities_response);
    identities_response =
        filter_identities_by_lastmodified(identities_response, args.inactive_days);

    info!(
        "Found a total of {} inactive identities.",
        identities_response.len()
    );

    dump_identities(&identities_response)
        .await
        .expect("Could not dump identities to file");

    if !args.dry_run {
        delete_identities(
            bearer_token,
            key_values.identityServiceUrl,
            key_values.accountId,
            &identities_response,
            args.concurrency,
        )
        .await
        .expect("Deletion failed");
    } else {
        info!("Dry Run, Aborting. To delete identities rerun with --dry-run false");
    }

    Ok(())
}
