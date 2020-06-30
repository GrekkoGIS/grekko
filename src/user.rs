use serde::export::fmt;
use serde::{Deserialize, Serialize};
use warp::{reject, Reply, Rejection};
use warp::reply::Response;
use jsonwebtoken::dangerous_unsafe_decode;
use crate::{Claims, redis_manager};
use chrono::{NaiveDateTime, Utc};

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
    let result = redis_manager::get::<User>("USERS", user.as_str());
    match result {
        None => Err(reject::custom(UserFail::new(user))),
        Some(res) => Ok(warp::reply::json(&res)),
    }
}

pub async fn set_user_details(token: String, user: User) -> Result<impl warp::Reply, Rejection> {
    let id = user.id.clone();
    let id = id.as_str();
    let result = redis_manager::set::<User>("USERS", id, user);
    match result {
        Some(value) => Ok(warp::reply::json(&String::from(value))),
        None => Err(reject()),
    }
}

pub async fn get_user_claims(
    token: String,
) -> Result<impl Reply, Rejection> {
    let tokens: Vec<&str> = token.split("Bearer ").collect();
    let token = tokens.get(1).unwrap().clone();
    let result = dangerous_unsafe_decode::<Claims>(&token);
    let claims = result.unwrap().claims;
    let uid = claims.uid.clone();
    validate_expiry(&claims)?;
    get_user_details(uid).await
}

fn validate_expiry(claims: &Claims) -> Result<(), Rejection> {
    let expiry_date_time = NaiveDateTime::from_timestamp(claims.exp, 0).timestamp();
    let now = Utc::now().naive_utc().timestamp();
    if expiry_date_time <= now {
        return Err(reject::not_found())
    } else {
        Ok(())
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
