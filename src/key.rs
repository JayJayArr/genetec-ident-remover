use serde::{Deserialize, Serialize};
#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyFile {
    pub accountId: String,
    pub key: Key,
    pub clientId: String,
    pub clientSecret: String,
    pub stsUrl: String,
    pub identityServiceUrl: String,
    pub principalServiceUrl: String,
    pub teamServiceUrl: String,
    pub roleServiceUrl: String,
    pub identityRequestServiceUrl: String,
    pub webhooksServiceUrl: String,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Key {
    pub kid: String,
    pub keyType: String,
    pub algorithm: String,
    pub exponent: String,
    pub modulus: String,
    pub d: String,
    pub dp: String,
    pub dq: String,
    pub inverseQ: String,
    pub p: String,
    pub q: String,
}
