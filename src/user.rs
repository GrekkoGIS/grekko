use failure::{Error, ResultExt};
use serde::export::fmt;
use serde::{Deserialize, Serialize};
use vrp_pragmatic::format::solution::Solution;
use warp::reply::{Json, Response};
use warp::{reject, Rejection, Reply};

use crate::auth;
use crate::redis_manager;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    uid: String,
    forward_geocoding: Vec<String>,
    reverse_geocoding: Vec<Vec<f64>>,
    routes: Vec<Solution>,
}

impl warp::reply::Reply for User {
    fn into_response(self) -> Response {
        Response::new(
            serde_json::to_string(&self)
                .expect("Unable to serialise User")
                .into(),
        )
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ForwardGeocoding: {:?}, ReverseGeocoding: {:?}, Routes: {:?}",
            self.forward_geocoding, self.reverse_geocoding, self.routes
        )
    }
}

impl User {
    pub async fn wrap_reply(self) -> Result<Json, Rejection> {
        let user = get_user_details(self.uid).await;
        match user {
            Ok(user) => Ok(warp::reply::json(&user)),
            Err(err) => Err(reject::reject()), //TODO
        }
    }

    pub fn add_route(self, route: Solution) -> User {
        let mut user = self.clone();
        user.routes.push(route);
        user
    }
}

#[derive(Debug)]
pub struct UserFail {
    message: String,
}

impl reject::Reject for UserFail {}

impl UserFail {
    pub fn new(id: String) -> UserFail {
        UserFail {
            message: format!("Unable to find a user with id `{}`", id),
        }
    }
}

pub async fn get_user(token: String) -> Result<User, Error> {
    let uid = get_id_from_token(token)
        .await
        .with_context(|err| format!("Failed to get uid from token err `{}`", err))?;
    log::trace!("User `{}` decoded from token", uid);

    get_user_details(uid).await
}

pub async fn get_user_details(uid: String) -> Result<User, Error> {
    let user = redis_manager::get::<User>("USERS", uid.as_str()).with_context(|err| {
        format!(
            "Failed to get user `{}` from table USERS err `{}`",
            uid, err
        )
    })?;
    Ok(user)
}

pub async fn set_user(user: User) -> Option<String> {
    redis_manager::set::<User>("USERS", &user.uid, user.clone())
}

pub async fn get_id_from_token(token: String) -> Result<String, Error> {
    let valid_jwt = auth::decode_token_unsafe(token.clone())
        .await
        .with_context(|err| format!("Failed to decode token err: `{}`", err))?;
    log::trace!("TokenData `{:?}` decoded", valid_jwt);
    Ok(valid_jwt.claims.uid)
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_get_user_claims() {}
}
