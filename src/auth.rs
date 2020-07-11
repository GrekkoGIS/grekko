use alcoholic_jwt::{token_kid, validate, ValidJWT, Validation, JWKS};
use failure::ResultExt;
use jsonwebtoken::TokenData;
use log::debug;
use serde::de::DeserializeOwned;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Claims {
    pub uid: String,
    pub sub: String,
}

pub(crate) async fn decode_token_unsafe(
    token: String,
) -> Result<TokenData<Claims>, failure::Error> {
    let token_index = 1;
    let token: Vec<&str> = token.split("Bearer ").collect();
    let token = token
        .get(token_index)
        .ok_or_else(|| failure::err_msg("Failed to get the token index"))?;

    log::debug!("Decoding token `{}`", token);
    Ok(jsonwebtoken::dangerous_insecure_decode(&token)?)
}

#[cfg(test)]
mod tests {}
