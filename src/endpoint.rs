use futures_util::{StreamExt, stream};
use indicatif::ProgressBar;
use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::{ClientId, ClientSecret, EmptyExtraTokenFields, StandardTokenResponse, TokenUrl};
use reqwest::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use tracing::{error, info};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IdentityResponse {
    identities: Vec<Value>,
    total_items: Option<u64>,
    continuation: Option<String>,
}

pub async fn get_bearer_token(
    client_id: String,
    client_secret: String,
    base_url: String,
) -> anyhow::Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
    info!("Authenticating with the Genetec API: {}", base_url);
    let url = format!("{}/connect/token", base_url);
    let oauth_client = BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_token_uri(TokenUrl::new(url)?);

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

    let identity_client = Client::new();
    let mut continuation = String::new();
    let mut identities = Vec::new();
    let mut total;
    let pb = ProgressBar::no_length();

    info!("Getting identities for AccountID {}", account_id);
    loop {
        let response = identity_client
            .get(&url)
            .bearer_auth(bearer_token)
            .query(&[("Take", 100)])
            .query(&[("Continuation", continuation)])
            .send()
            .await?
            .json::<IdentityResponse>()
            .await?;
        identities.extend(response.identities);
        total = response
            .total_items
            .expect("Could not find total value in response");
        pb.set_length(total);
        pb.set_position(identities.len().try_into().unwrap());
        match response.continuation {
            Some(new_continuation_token) => continuation = new_continuation_token,
            None => break,
        }
    }
    pb.finish_with_message("Done!");

    Ok(identities)
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
            delete_identity_callback(
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
async fn delete_identity_callback(
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

pub async fn delete_pictures(
    bearer_token: &str,
    identity_base_url: String,
    account_id: String,
    identities: &Vec<Value>,
    concurrency: usize,
) -> anyhow::Result<()> {
    info!("Deleting all pictures for AccountID {}...", account_id);

    let client = Client::new();
    stream::iter(identities)
        .for_each_concurrent(concurrency, |identity_id| {
            delete_pictures_callback(
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

async fn delete_pictures_callback(
    client: &reqwest::Client,
    base_url: String,
    account_id: String,
    identity: &Value,
    bearer_token: &str,
) {
    let identity_id = identity.get("identityId").unwrap().as_str().unwrap();
    let url = format!(
        "{}/api/v4/accounts/{}/identities/{}/picture",
        base_url, account_id, identity_id
    );

    match client.delete(url).bearer_auth(bearer_token).send().await {
        Ok(res) => {
            if res.status() != StatusCode::OK {
                error!(
                    "Error deleting picture from {}: {}",
                    identity_id,
                    res.text()
                        .await
                        .expect("Could not get http response text from bad request")
                );
            } else {
                info!("successful deletion of picture from {}", identity_id);
            }
        }

        Err(e) => error!("Error deleting picture from {}: {}", identity_id, e),
    };
}

#[cfg(test)]
mod tests {
    use oauth2::TokenResponse;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path, query_param},
    };

    use crate::endpoint::{
        delete_identity_callback, delete_pictures_callback, get_all_identities, get_bearer_token,
    };

    #[tokio::test]
    async fn test_bearer_token_request() {
        let mock_server = MockServer::start().await;
        Mock::given(path("/connect/token"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
              "access_token":"MTQ0NjJkZmQ5OTM2NDE1ZTZjNGZmZjI3",
              "token_type":"Bearer",
              "expires_in":3600,
              "refresh_token":"IwOGYzYTlmM2YxOTQ5MGE3YmNmMDFkNTVk",
              "scope":"create"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let token = get_bearer_token(
            "client_id".to_string(),
            "client_secret".to_string(),
            mock_server.uri(),
        )
        .await;
        assert!(token.is_ok());
    }

    #[tokio::test]
    async fn test_get_identities_request() {
        let mock_server = MockServer::start().await;
        Mock::given(path("/api/v4/accounts/accountID/identities"))
            .and(method("GET"))
            .and(query_param("Take", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!(
            {
                "identities":[
                    {
                        "accountId": "abcdefgdh-fcfc-0000-abdc-afafafafafafaf",
                        "companyData": {
                        "approvers": []
                        },
                        "createdBy": "SystemService",
                        "creationDateUtc": "2025-07-02T14:52:39.0438424Z",
                        "displayName": "John Doe",
                        "eTag": "2",
                        "email": "john.doe@example.com",
                        "firstName": "John",
                        "hasLicensedVehicles": false,
                        "hasVehicles": false,
                        "identityId": "d2c68f36-fb4e-4606-b831-617f7ab06094",
                        "identityType": "Employee",
                        "isDeleted": false,
                        "isSCSaaS": true,
                        "lastModificationDateUtc": "2026-02-27T09:07:19.1960627Z",
                        "lastModifiedBy": "phtephen@example.com",
                        "lastModifiedByIdentityId": "bc1b3d75-f2a5-4aee-8c13-ad1dfe3b54cb",
                        "lastModifiedByPrincipalType": "User",
                        "lastName": "Doe",
                        "ordinal": 2,
                        "privateData": {},
                        "status": "Inactive",
                        "systemData": {
                        "customFields": [],
                        "horizonId": "5a56e94b92964d6da3d57258508b42e7",
                        "provisioningAttributes": [],
                        "resourceFilters": []
                        }
                    },
                ],
                "totalItems": 1,
            }
            )))
            .expect(1)
            .mount(&mock_server)
            .await;

        let response =
            get_all_identities("token", mock_server.uri(), "accountID".to_string()).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_get_identities_request_with_pagination() {
        let mock_server = MockServer::start().await;
        //pagination mock
        Mock::given(path("/api/v4/accounts/accountID/identities"))
            .and(method("GET"))
            .and(query_param("Take", "100"))
            .and(query_param("Continuation", ""))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!(
            {
                "identities":[
                    {
                        "accountId": "abcdefgdh-fcfc-0000-abdc-afafafafafafaf",
                        "companyData": {
                        "approvers": []
                        },
                        "createdBy": "SystemService",
                        "creationDateUtc": "2025-07-02T14:52:39.0438424Z",
                        "displayName": "John Doe",
                        "eTag": "2",
                        "email": "john.doe@example.com",
                        "firstName": "John",
                        "hasLicensedVehicles": false,
                        "hasVehicles": false,
                        "identityId": "d2c68f36-fb4e-4606-b831-617f7ab06094",
                        "identityType": "Employee",
                        "isDeleted": false,
                        "isSCSaaS": true,
                        "lastModificationDateUtc": "2026-02-27T09:07:19.1960627Z",
                        "lastModifiedBy": "phtephen@example.com",
                        "lastModifiedByIdentityId": "bc1b3d75-f2a5-4aee-8c13-ad1dfe3b54cb",
                        "lastModifiedByPrincipalType": "User",
                        "lastName": "Doe",
                        "ordinal": 2,
                        "privateData": {},
                        "status": "Inactive",
                        "systemData": {
                        "customFields": [],
                        "horizonId": "5a56e94b92964d6da3d57258508b42e7",
                        "provisioningAttributes": [],
                        "resourceFilters": []
                        }
                    },
                ],
                "totalItems": 1,
                "continuation": "continuationString"
            }
            )))
            .expect(1)
            .mount(&mock_server)
            .await;
        // Normal mock
        Mock::given(path("/api/v4/accounts/accountID/identities"))
            .and(method("GET"))
            .and(query_param("Take", "100"))
            .and(query_param("Continuation", "continuationString"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!(
            {
                "identities":[
                    {
                        "accountId": "abcdefgdh-fcfc-0000-abdc-afafafafafafaf",
                        "companyData": {
                        "approvers": []
                        },
                        "createdBy": "SystemService",
                        "creationDateUtc": "2025-07-02T14:52:39.0438424Z",
                        "displayName": "John Doe",
                        "eTag": "2",
                        "email": "john.doe@example.com",
                        "firstName": "John",
                        "hasLicensedVehicles": false,
                        "hasVehicles": false,
                        "identityId": "d2c68f36-fb4e-4606-b831-617f7ab06094",
                        "identityType": "Employee",
                        "isDeleted": false,
                        "isSCSaaS": true,
                        "lastModificationDateUtc": "2026-02-27T09:07:19.1960627Z",
                        "lastModifiedBy": "phtephen@example.com",
                        "lastModifiedByIdentityId": "bc1b3d75-f2a5-4aee-8c13-ad1dfe3b54cb",
                        "lastModifiedByPrincipalType": "User",
                        "lastName": "Doe",
                        "ordinal": 2,
                        "privateData": {},
                        "status": "Inactive",
                        "systemData": {
                        "customFields": [],
                        "horizonId": "5a56e94b92964d6da3d57258508b42e7",
                        "provisioningAttributes": [],
                        "resourceFilters": []
                        }
                    },
                ],
                "totalItems": 1,
            }
            )))
            .expect(1)
            .mount(&mock_server)
            .await;

        let response =
            get_all_identities("token", mock_server.uri(), "accountID".to_string()).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_delete_identities_request() {
        let mock_server = MockServer::start().await;
        Mock::given(path(
            "/api/v4/accounts/accountID/identities/d2c68f36-fb4e-4606-b831-617f7ab06094",
        ))
        .and(method("DELETE"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;
        Mock::given(path("/connect/token"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
              "access_token":"MTQ0NjJkZmQ5OTM2NDE1ZTZjNGZmZjI3",
              "token_type":"Bearer",
              "expires_in":3600,
              "refresh_token":"IwOGYzYTlmM2YxOTQ5MGE3YmNmMDFkNTVk",
              "scope":"create"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;
        let token = get_bearer_token(
            "client_id".to_string(),
            "client_secret".to_string(),
            mock_server.uri(),
        )
        .await;

        delete_identity_callback(
            &reqwest::Client::new(),
            mock_server.uri(),
            "accountID".to_string(),
            &json!(
                {
                    "accountId": "abcdefgdh-fcfc-0000-abdc-afafafafafafaf",
                    "companyData": {
                    "approvers": []
                    },
                    "createdBy": "SystemService",
                    "creationDateUtc": "2025-07-02T14:52:39.0438424Z",
                    "displayName": "John Doe",
                    "eTag": "2",
                    "email": "john.doe@example.com",
                    "firstName": "John",
                    "hasLicensedVehicles": false,
                    "hasVehicles": false,
                    "identityId": "d2c68f36-fb4e-4606-b831-617f7ab06094",
                    "identityType": "Employee",
                    "isDeleted": false,
                    "isSCSaaS": true,
                    "lastModificationDateUtc": "2026-02-27T09:07:19.1960627Z",
                    "lastModifiedBy": "phtephen@example.com",
                    "lastModifiedByIdentityId": "bc1b3d75-f2a5-4aee-8c13-ad1dfe3b54cb",
                    "lastModifiedByPrincipalType": "User",
                    "lastName": "Doe",
                    "ordinal": 2,
                    "privateData": {},
                    "status": "Inactive",
                    "systemData": {
                    "customFields": [],
                    "horizonId": "5a56e94b92964d6da3d57258508b42e7",
                    "provisioningAttributes": [],
                    "resourceFilters": []
                    }
                }
            ),
            token.unwrap().access_token().clone().into_secret().as_str(),
        )
        .await;
    }

    #[tokio::test]
    async fn test_delete_picture_request() {
        let mock_server = MockServer::start().await;
        Mock::given(path(
            "/api/v4/accounts/accountID/identities/d2c68f36-fb4e-4606-b831-617f7ab06094/picture",
        ))
        .and(method("DELETE"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;
        Mock::given(path("/connect/token"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
              "access_token":"MTQ0NjJkZmQ5OTM2NDE1ZTZjNGZmZjI3",
              "token_type":"Bearer",
              "expires_in":3600,
              "refresh_token":"IwOGYzYTlmM2YxOTQ5MGE3YmNmMDFkNTVk",
              "scope":"create"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;
        let token = get_bearer_token(
            "client_id".to_string(),
            "client_secret".to_string(),
            mock_server.uri(),
        )
        .await;

        delete_pictures_callback(
            &reqwest::Client::new(),
            mock_server.uri(),
            "accountID".to_string(),
            &json!(
                {
                    "accountId": "abcdefgdh-fcfc-0000-abdc-afafafafafafaf",
                    "companyData": {
                    "approvers": []
                    },
                    "createdBy": "SystemService",
                    "creationDateUtc": "2025-07-02T14:52:39.0438424Z",
                    "displayName": "John Doe",
                    "eTag": "2",
                    "email": "john.doe@example.com",
                    "firstName": "John",
                    "hasLicensedVehicles": false,
                    "hasVehicles": false,
                    "identityId": "d2c68f36-fb4e-4606-b831-617f7ab06094",
                    "identityType": "Employee",
                    "isDeleted": false,
                    "isSCSaaS": true,
                    "lastModificationDateUtc": "2026-02-27T09:07:19.1960627Z",
                    "lastModifiedBy": "phtephen@example.com",
                    "lastModifiedByIdentityId": "bc1b3d75-f2a5-4aee-8c13-ad1dfe3b54cb",
                    "lastModifiedByPrincipalType": "User",
                    "lastName": "Doe",
                    "ordinal": 2,
                    "privateData": {},
                    "status": "Inactive",
                    "systemData": {
                    "customFields": [],
                    "horizonId": "5a56e94b92964d6da3d57258508b42e7",
                    "provisioningAttributes": [],
                    "resourceFilters": []
                    }
                }
            ),
            token.unwrap().access_token().clone().into_secret().as_str(),
        )
        .await;
    }
}
