use crate::auth::validate_token;
use crate::{redis_manager, Claims};
use alcoholic_jwt::token_kid;
use chrono::{NaiveDateTime, Utc};
use jsonwebtoken::{
    dangerous_unsafe_decode, decode, decode_header, Algorithm, DecodingKey, TokenData, Validation,
};
use serde::export::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use warp::reply::Response;
use warp::{reject, Rejection, Reply};

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

pub async fn set_user_details(token: String, user: User) -> Result<impl warp::Reply, Rejection> {
    decode_token(token).await;
    let id = user.id.clone();
    let id = id.as_str();
    let result = redis_manager::set::<User>("USERS", id, user);
    match result {
        Some(value) => Ok(warp::reply::json(&value)),
        None => Err(reject()),
    }
}

pub async fn get_user_claims(token: String) -> Result<impl Reply, Rejection> {
    let uid = decode_token(token).await.get("uid").unwrap().to_string();
    get_user_details(uid).await
}

async fn decode_token(token: String) -> Value {
    let token: Vec<&str> = token.split("Bearer ").collect();
    let token = token.get(1).expect("Failed to get the token index").clone();

    let token_data = validate_token(token.to_string())
        .await
        .expect("Failed to unwrap the token");

    token_data.claims
}

fn decode_admin(token: String) {
    let tokens: Vec<&str> = token.split("Bearer ").collect();
    let token = tokens.get(1).unwrap().clone();
    let result = dangerous_unsafe_decode::<Claims>(&token);
    let claims = result.unwrap().claims;

    let mut validation = Validation::default();
    validation.set_audience(&["api://default"]);
    validation.leeway = 60000;
    validation.sub = Some(String::from("awesomealpineibex@gmail.com"));
    validation.validate_exp = false;
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
