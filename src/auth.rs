use jsonwebtoken::TokenData;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Claims {
    pub uid: String,
    pub sub: String,
}

pub(crate) async fn decode_token_unsafe(
    token: String,
) -> Result<TokenData<Claims>, failure::Error> {
    log::trace!("Decoding token {}", token);
    let token_index = 1;
    let token: Vec<&str> = token.split("Bearer ").collect();
    let token = token
        .get(token_index)
        .ok_or_else(|| failure::err_msg("Failed to get the token index"))?;

    Ok(jsonwebtoken::dangerous_insecure_decode(&token)?)
}

#[cfg(test)]
mod tests {}
