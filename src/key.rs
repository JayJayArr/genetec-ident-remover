use serde::Deserialize;
#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct KeyFile {
    accountId: String,
    key: Key,
    clientId: String,
    clientSecret: String,
    stsUrl: String,
    identityServiceUrl: String,
    principalServiceUrl: String,
    teamServiceUrl: String,
    roleServiceUrl: String,
    identityRequestServiceUrl: String,
    webhooksServiceUrl: String,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Key {
    kid: String,
    keyType: String,
    algorithm: String,
    exponent: String,
    modulus: String,
    d: String,
    dp: String,
    dq: String,
    inverseQ: String,
    p: String,
    q: String,
}
