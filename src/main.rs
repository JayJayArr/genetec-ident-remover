use crate::key::KeyFile;
use crate::telemetry::init_tracing;
use clap::Parser;
use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::{ClientId, ClientSecret, EmptyExtraTokenFields, StandardTokenResponse, TokenUrl};
use oauth2::{TokenResponse, reqwest};
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

    let tokenresponse = get_api_token(
        key_values.clientId,
        key_values.clientSecret,
        format!("{}/connect/token", key_values.stsUrl),
    )
    .await?;

    info!("Token obtained: {}", tokenresponse.access_token().secret());
    Ok(())
}

async fn get_api_token(
    client_id: String,
    client_secret: String,
    endpoint: String,
) -> anyhow::Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
    info!("Authenticating with the Genetec API: {}", endpoint);
    let client = BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_token_uri(TokenUrl::new(endpoint)?);

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let token_result: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType> = client
        .exchange_client_credentials()
        .request_async(&http_client)
        .await?;

    info!("Authentication successful");
    Ok(token_result)
}
