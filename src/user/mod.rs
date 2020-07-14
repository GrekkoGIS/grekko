use failure::{Error, ResultExt};
use vrp_pragmatic::format::solution::Solution;

pub mod structs;

use crate::auth;
use crate::redis_manager::get_manager;
use crate::redis_manager::json_cache_manager::JsonCacheManager;
use crate::user::structs::User;

pub async fn get_id_from_token(token: String) -> Result<String, Error> {
    let valid_jwt = auth::decode_token_unsafe(token.clone())
        .await
        .with_context(|err| format!("Failed to decode token err: `{}`", err))?;
    log::trace!("TokenData `{:?}` decoded", valid_jwt);
    Ok(valid_jwt.claims.uid)
}

pub async fn get_user(token: String) -> Result<User, Error> {
    let uid = get_id_from_token(token)
        .await
        .with_context(|err| format!("Failed to get uid from token err `{}`", err))?;
    log::trace!("User `{}` decoded from token", uid);

    get_user_details(uid).await
}

pub async fn get_user_details(uid: String) -> Result<User, Error> {
    let user = get_manager()
        .get_json(uid.as_str(), None)
        .with_context(|err| {
            format!(
                "Failed to get user `{}` from table USERS err `{}`",
                uid, err
            )
        })?;
    Ok(user)
}

pub async fn set_user(user: User) -> Option<String> {
    get_manager().set_json(&user.uid, None, user.clone())
}

pub async fn append_user_route(user: User, route: &Solution) -> Option<String> {
    get_manager().append_json(&user.uid, ".routes", &route)
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_get_user_claims() {}
}
