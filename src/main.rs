use std::process::exit;

use crate::key::KeyFile;
use crate::telemetry::init_tracing;
use clap::Parser;
use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::{ClientId, ClientSecret, EmptyExtraTokenFields, StandardTokenResponse, TokenUrl};
use oauth2::{TokenResponse, reqwest};
use reqwest::Client;
use serde_json::Value;
use tracing::info;
use tracing::warn;
mod key;
mod telemetry;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// key file from Genetec to authenticate
    #[arg(short)]
    keyfile: String,

    /// Display users, but do not delete
    #[arg(long, default_value_t = false)]
    dry_run: bool,
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

    info!("Token obtained: {}", tokenresponse.access_token().secret());

    let mut identities_response = get_all_identities(
        tokenresponse.access_token().secret(),
        key_values.identityServiceUrl,
        key_values.accountId,
    )
    .await?;

    identities_response = filter_identities_by_status(identities_response);

    let mut relevant_identities: Vec<String> = vec![];
    for identity in identities_response {
        let id = identity.get("identityId").unwrap().to_string();
        let email = identity.get("email").unwrap_or_default().to_string();
        let lastmodified = identity
            .get("lastModificationDateUtc")
            .unwrap_or_default()
            .to_string();
        dbg!(&lastmodified);

        info!(
            "Inactive Identity found: Id: {}, email: {}, lastmodified: {}",
            id, email, lastmodified
        );
        relevant_identities.push(id);
    }

    info!(
        "Found a total of {} inactive identities.",
        relevant_identities.len()
    );

    if args.dry_run {
        info!("Dry Run, Aborting. To delete identities rerun without --dry-run");
        exit(0);
    }
    Ok(())
}

async fn get_bearer_token(
    client_id: String,
    client_secret: String,
    endpoint: String,
) -> anyhow::Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
    info!("Authenticating with the Genetec API: {}", endpoint);
    let oauth_client = BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_token_uri(TokenUrl::new(endpoint)?);

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let token_result: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType> = oauth_client
        .exchange_client_credentials()
        .request_async(&http_client)
        .await?;

    info!("Authentication successful");
    Ok(token_result)
}

async fn get_all_identities(
    bearer_token: &str,
    identity_base_url: String,
    account_id: String,
) -> anyhow::Result<Vec<Value>> {
    let url = format!(
        "{}/api/v4/accounts/{}/identities",
        identity_base_url, account_id
    );

    info!(
        "Getting identities for AccountID {} from {}",
        account_id, url
    );

    let identity_client = Client::new();
    let response = identity_client
        .get(url)
        .bearer_auth(bearer_token)
        .send()
        .await?;
    let body = response.text().await?;
    let json: serde_json::Value = serde_json::from_str(&body)?;
    Ok(json
        .get("identities")
        .expect("Could not find field \"identities\" in the json response")
        .as_array()
        .expect("Could not convert the Identities in an array")
        .clone())
}

fn filter_identities_by_status(identities: Vec<Value>) -> Vec<Value> {
    info!("Filtering {} identities by Status...", identities.len());
    let identities: Vec<Value> = identities
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
        identities.len()
    );
    identities
}
