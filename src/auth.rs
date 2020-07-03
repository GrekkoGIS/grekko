use alcoholic_jwt::{token_kid, validate, ValidJWT, Validation, JWK, JWKS};
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthKey {
    pub kty: String,
    pub status: String,
    pub alg: String,
    pub kid: String,
    #[serde(rename = "use")]
    pub use_field: String,
    #[serde(rename = "_links")]
    pub links: Links,
    pub e: String,
    pub n: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    #[serde(rename = "self")]
    pub self_field: SelfField,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfField {
    pub href: String,
    pub hints: Hints,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hints {
    pub allow: Vec<String>,
}

pub async fn validate_token(token: String) -> Result<ValidJWT, failure::Error> {
    let keys = get_jwks().await?;

    let kid = token_kid(&token)
        .expect("Failed to decode token headers")
        .expect("No 'kid' claim present in token");

    let jwk = keys.find(&kid).expect("Specified key not found in set");

    let validations = vec![Validation::NotExpired, Validation::SubjectPresent];

    let res = validate(&token, jwk, validations).expect("Token validation has failed!");

    Ok(res)
}

pub async fn get_jwks() -> Result<JWKS, failure::Error> {
    let api_key = env!("OKTA_API_KEY").to_string();
    let mut api_key = String::from("SSWS ") + &api_key;
    const URL: &str = "https://dev-201460.okta.com/api/v1";
    let client = reqwest::Client::new();

    let mut url = String::from(URL);
    url.push_str("/authorizationServers");
    url.push_str("/default/credentials/keys");

    let response = client
        .get(&url)
        .header("Authorization", &api_key)
        .send()
        .await?
        .text()
        .await?;

    let mut response_sanitized = String::new();
    response_sanitized.push_str("{");
    response_sanitized.push_str("\"keys\":");
    response_sanitized.push_str(&response);
    response_sanitized.push_str("}");

    let body: JWKS = serde_json::from_str::<JWKS>(&response_sanitized)?;

    debug!("Got signing keys: {:?}", body);
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_signing_key() {
        let res = get_jwks().await.unwrap();
        println!("{:?}", res)
    }
}
