use std::process::exit;

use crate::endpoint::{delete_identities, delete_pictures, get_all_identities, get_bearer_token};
use crate::filter::{
    dump_identities, filter_identities_by_lastmodified, filter_identities_by_status,
};
use crate::key::KeyFile;
use crate::telemetry::init_tracing;
use clap::Parser;
use clap::Subcommand;
use oauth2::TokenResponse;
use tracing::info;
use tracing::warn;
mod endpoint;
mod filter;
mod key;
mod telemetry;

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command()]
    ListInactiveIdentities {
        #[arg(short)]
        keyfile: String,

        /// Minimum Inactivity Period in days for an `Identity` to be deleted
        #[arg(short, long, default_value_t = 90)]
        inactive_days: i64,
    },
    PurgeInactiveIdentities {
        #[arg(short)]
        keyfile: String,

        /// Minimum Inactivity Period in days for an `Identity` to be deleted
        #[arg(short, long, default_value_t = 90)]
        inactive_days: i64,

        /// Number of concurrent requests when deleting the Identities
        #[arg(short, long, default_value_t = 10)]
        concurrency: usize,
    },
    PurgePictures {
        #[arg(short)]
        keyfile: String,

        /// Number of concurrent requests when deleting the Identities
        #[arg(short, long, default_value_t = 10)]
        concurrency: usize,
    },
}

#[tokio::main]

async fn main() -> anyhow::Result<()> {
    init_tracing()?;
    let args = Cli::parse();
    match args.command {
        Commands::ListInactiveIdentities {
            keyfile,
            inactive_days,
        } => {
            let key_values: KeyFile = get_keyfile(keyfile).await?;
            info!(
                "Displaying all inactive identities for {}",
                key_values.accountId
            );
            let tokenresponse = get_bearer_token(
                key_values.clientId,
                key_values.clientSecret,
                key_values.stsUrl,
            )
            .await?;
            let bearer_token = tokenresponse.access_token().secret();

            let mut identities_response = get_all_identities(
                bearer_token,
                key_values.identityServiceUrl.clone(),
                key_values.accountId.clone(),
            )
            .await?;

            //Apply filters
            //TODO: make this configurable via flags
            identities_response = filter_identities_by_status(identities_response);
            identities_response =
                filter_identities_by_lastmodified(identities_response, inactive_days);

            info!(
                "Found a total of {} inactive identities.",
                identities_response.len()
            );

            dump_identities(&identities_response)
                .await
                .expect("Could not dump identities to file");
            info!(
                "To delete the unused identities please use the subcommand purge-inactive-identies"
            );
        }

        Commands::PurgeInactiveIdentities {
            keyfile,
            inactive_days,
            concurrency,
        } => {
            let key_values: KeyFile = get_keyfile(keyfile).await?;
            warn!(
                "Deleting inactive identies from system {}",
                key_values.accountId
            );
            get_confirmation();
            let tokenresponse = get_bearer_token(
                key_values.clientId,
                key_values.clientSecret,
                key_values.stsUrl,
            )
            .await?;
            let bearer_token = tokenresponse.access_token().secret();

            let mut identities_response = get_all_identities(
                bearer_token,
                key_values.identityServiceUrl.clone(),
                key_values.accountId.clone(),
            )
            .await?;

            //Apply filters
            //TODO: make this configurable via flags
            identities_response = filter_identities_by_status(identities_response);
            identities_response =
                filter_identities_by_lastmodified(identities_response, inactive_days);

            info!(
                "Found a total of {} inactive identities.",
                identities_response.len()
            );

            dump_identities(&identities_response)
                .await
                .expect("Could not dump identities to file");

            delete_identities(
                bearer_token,
                key_values.identityServiceUrl,
                key_values.accountId,
                &identities_response,
                concurrency,
            )
            .await
            .expect("Deletion failed");
        }

        Commands::PurgePictures {
            keyfile,
            concurrency,
        } => {
            let key_values: KeyFile = get_keyfile(keyfile).await?;
            warn!("Deleting all pictures from system {}", key_values.accountId);
            get_confirmation();
            let tokenresponse = get_bearer_token(
                key_values.clientId,
                key_values.clientSecret,
                key_values.stsUrl,
            )
            .await?;
            let bearer_token = tokenresponse.access_token().secret();

            let identities_response = get_all_identities(
                bearer_token,
                key_values.identityServiceUrl.clone(),
                key_values.accountId.clone(),
            )
            .await?;

            info!("Found a total of {} identities.", identities_response.len());

            dump_identities(&identities_response)
                .await
                .expect("Could not dump identities to file");

            delete_pictures(
                bearer_token,
                key_values.identityServiceUrl,
                key_values.accountId,
                &identities_response,
                concurrency,
            )
            .await
            .expect("Deletion failed");
        }
    }
    Ok(())
}

async fn get_keyfile(filename: String) -> anyhow::Result<KeyFile> {
    let file = tokio::fs::read_to_string(filename).await.unwrap();

    let keyfile = serde_json::from_str(file.as_str())?;
    Ok(keyfile)
}

fn get_confirmation() {
    let mut input = String::new();

    println!("Are you sure you want to do this? y/N");

    std::io::stdin().read_line(&mut input).unwrap();
    if input.trim() != "y".to_string() {
        exit(1)
    }
}

#[cfg(test)]
mod tests {
    use crate::get_keyfile;

    #[tokio::test]
    async fn test_dummy_keyfile_is_parsed_correctly() {
        assert!(get_keyfile("./key-dummy.json".to_string()).await.is_ok())
    }
}
