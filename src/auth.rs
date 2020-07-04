use alcoholic_jwt::{token_kid, validate, ValidJWT, Validation, ValidationError, JWK, JWKS};
use failure::{Fail, ResultExt};
use log::debug;
use serde::{Deserialize, Serialize};

pub async fn validate_token(token: String) -> Result<ValidJWT, failure::Error> {
    let keys = get_jwks().await?;

    let kid = token_kid(&token)
        .expect("Failed to decode token headers")
        .expect("No 'kid' claim present in token");

    let jwk = keys
        .find(&kid)
        .ok_or_else(|| failure::err_msg("Specified key not found in set"))?;

    let validations = vec![Validation::NotExpired, Validation::SubjectPresent];

    let res = validate(&token, jwk, validations);

    match res {
        Ok(res) => Ok(res),
        Err(err) => Err(failure::err_msg(format!(
            "Failed to validate JWT: {:?}",
            err
        ))),
    }
}

// TODO: don't do this alot
pub async fn get_jwks() -> Result<JWKS, failure::Error> {
    let api_key = env!("OKTA_API_KEY").to_string();
    let api_key = String::from("SSWS ") + &api_key;
    const URL: &str = "https://dev-201460.okta.com/api/v1";
    let client = reqwest::Client::new();

    let mut url = String::from(URL);
    url.push_str("/authorizationServers");
    url.push_str("/default/credentials/keys");

    let response = client
        .get(&url)
        .header("Authorization", &api_key)
        .send()
        .await
        .with_context(|_| "Failed to get keys")?
        .text()
        .await
        .with_context(|_| "Failed to read text")?;

    let mut response_sanitized = String::new();
    response_sanitized.push_str("{");
    response_sanitized.push_str("\"keys\":");
    response_sanitized.push_str(&response);
    response_sanitized.push_str("}");

    let body: JWKS = serde_json::from_str::<JWKS>(&response_sanitized)?;

    debug!("Got signing keys: {:?}", body);
    Ok(body)
}

pub(crate) async fn decode_token(token: String) -> Result<ValidJWT, failure::Error> {
    let token_index = 1;
    let token: Vec<&str> = token.split("Bearer ").collect();
    let token = token
        .get(token_index)
        .ok_or_else(|| failure::err_msg("Failed to get the token index"))?;

    let token_data = validate_token(token.to_string())
        .await
        .with_context(|_| "Failed to unwrap the token")?;

    Ok(token_data)
}
pub(crate) async fn get_uid(token_data: ValidJWT) -> Result<String, failure::Error> {
    let uid = token_data
        .claims
        .get("uid")
        .ok_or_else(|| failure::err_msg("uid could not be found in jwk"))?
        .to_string();
    Ok(uid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_signing_key() {}
}
