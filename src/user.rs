use crate::auth::validate_token;
use crate::redis_manager;
use alcoholic_jwt::{token_kid, ValidJWT};
use chrono::{NaiveDateTime, Utc};
use failure::ResultExt;
use serde::export::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use warp::reply::Response;
use warp::{reject, Error, Rejection, Reply};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,
    forward_geocoding: Vec<String>,
    reverse_geocoding: Vec<Vec<f64>>,
    simple_routes: Vec<String>,
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
            "ID: {} ForwardGeocoding: {:?}, ReverseGeocoding: {:?}, SimpleRoutes: {:?}",
            self.id, self.forward_geocoding, self.reverse_geocoding, self.simple_routes
        )
    }
}

pub async fn get_user_details(user: String) -> Result<impl warp::Reply, Rejection> {
    let uid = decode_token(user.clone()).await;
    let result = redis_manager::get::<User>("USERS", user.as_str());
    match result {
        None => Err(reject::custom(UserFail::new(user))),
        Some(res) => Ok(warp::reply::json(&res)),
    }
}

pub async fn set_user_details(token: String, user: User) -> Result<impl Reply, Rejection> {
    decode_token(token).await;
    let id = user.id.clone();
    let id = id.as_str();
    let result = redis_manager::set::<User>("USERS", id, user);
    match result {
        Some(value) => Ok(warp::reply::json(&value)),
        None => Err(reject::reject()),
    }
}

pub async fn get_user_claims(token: String) -> Result<impl Reply, Rejection> {
    let valid_jwt = decode_token(token).await.or_else(|err| {
        log::error!("{:?}", err);
        Err(warp::reject())
    })?;

    let uid = get_uid(valid_jwt).await.or_else(|err| {
        log::error!("{:?}", err);
        Err(warp::reject())
    })?;

    get_user_details(uid).await
}

async fn decode_token(token: String) -> Result<ValidJWT, failure::Error> {
    let token: Vec<&str> = token.split("Bearer ").collect();
    let token = token
        .get(1)
        .ok_or_else(|| failure::err_msg("Failed to get the token index"))?;

    let token_data = validate_token(token.to_string())
        .await
        .with_context(|_| "Failed to unwrap the token")?;

    Ok(token_data)
}
async fn get_uid(token_data: ValidJWT) -> Result<String, failure::Error> {
    let uid = token_data
        .claims
        .get("uid")
        .ok_or_else(|| failure::err_msg("uid could not be found in jwk"))?
        .to_string();
    Ok(uid)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_user_claims() {}
}
