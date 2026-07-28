use crate::key::KeyFile;
use oauth2::basic::BasicClient;
use oauth2::http::{HeaderMap, header};
use oauth2::reqwest;
use oauth2::{ClientId, ClientSecret, TokenUrl};
mod key;

#[tokio::main]

async fn main() -> anyhow::Result<()> {
    let file =
        tokio::fs::read_to_string("key-94e25400-f2ce-42a0-a9b5-44973aa372b9-rietdorf_test.json")
            .await
            .unwrap();

    let key_values: KeyFile = serde_json::from_str(file.as_str())?;
    println!("{:?}", key_values);

    let client = BasicClient::new(ClientId::new(key_values.clientId))
        .set_client_secret(ClientSecret::new(key_values.clientSecret))
        .set_token_uri(TokenUrl::new(format!(
            "{}/connect/token",
            key_values.stsUrl
        ))?);

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let token_result = client
        .exchange_client_credentials()
        .request_async(&http_client)
        .await?;

    println!("{:?}", token_result);
    Ok(())
}
