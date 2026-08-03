use futures_util::{StreamExt, stream};
use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::{ClientId, ClientSecret, EmptyExtraTokenFields, StandardTokenResponse, TokenUrl};
use reqwest::Client;
use reqwest::StatusCode;
use serde_json::Value;
use tracing::{error, info};

pub async fn get_bearer_token(
    client_id: String,
    client_secret: String,
    endpoint: String,
) -> anyhow::Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
    info!("Authenticating with the Genetec API: {}", endpoint);
    let oauth_client = BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_token_uri(TokenUrl::new(endpoint)?);

    let http_client = oauth2::reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(oauth2::reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let token_result: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType> = oauth_client
        .exchange_client_credentials()
        .request_async(&http_client)
        .await?;

    info!("Authentication successful");
    Ok(token_result)
}

pub async fn get_all_identities(
    bearer_token: &str,
    identity_base_url: String,
    account_id: String,
) -> anyhow::Result<Vec<Value>> {
    let url = format!(
        "{}/api/v4/accounts/{}/identities",
        identity_base_url, account_id
    );

    info!("Getting identities for AccountID {}", account_id);

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
pub async fn delete_identities(
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
            delete_callback(
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
async fn delete_callback(
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
