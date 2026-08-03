use crate::filter::{filter_identities_by_lastmodified, filter_identities_by_status};
use crate::key::KeyFile;
use crate::telemetry::init_tracing;
use ::reqwest::StatusCode;
use chrono::Local;
use clap::Parser;
use futures_util::{StreamExt, stream};
use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::{ClientId, ClientSecret, EmptyExtraTokenFields, StandardTokenResponse, TokenUrl};
use oauth2::{TokenResponse, reqwest};
use reqwest::Client;
use serde_json::Value;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::warn;
use tracing::{error, info};
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

async fn delete_identities(
    bearer_token: &str,
    identity_base_url: String,
    account_id: String,
    identities: &Vec<Value>,
    concurrency: usize,
) -> anyhow::Result<()> {
    info!("Deleting identities for AccountID {}...", account_id);

    let client = Client::new();
    stream::iter(identities)
        .for_each_concurrent(concurrency, |identity_id| {
            callback(
                &client,
                identity_base_url.clone(),
                account_id.clone(),
                identity_id,
                bearer_token,
            )
        })
        .await;
    Ok(())
}

async fn dump_identities(identities: &Vec<Value>) -> anyhow::Result<()> {
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
async fn callback(
    client: &reqwest::Client,
    base_url: String,
    account_id: String,
    identity: &Value,
    bearer_token: &str,
) {
    let identity_id = identity.get("identityId").unwrap().as_str().unwrap();
    let etag = identity.get("eTag").unwrap_or_default().as_str().unwrap();
    let url = format!(
        "{}/api/v4/accounts/{}/identities/{}?eTag={}",
        base_url, account_id, identity_id, etag
    );

    match client.delete(url).bearer_auth(bearer_token).send().await {
        Ok(res) => {
            if res.status() != StatusCode::OK {
                error!(
                    "Error deleting {}: {}",
                    identity_id,
                    res.text()
                        .await
                        .expect("Could not get http response text from bad request")
                );
            } else {
                info!("successful deletion of {}", identity_id);
            }
        }

        Err(e) => error!("Error deleting {}: {}", identity_id, e),
    };
}
