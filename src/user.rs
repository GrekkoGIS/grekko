use crate::auth;
use crate::redis_manager;
use serde::export::fmt;
use serde::{Deserialize, Serialize};
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

pub async fn get_user_details(user: String) -> Result<impl warp::Reply, Rejection> {
    let result = redis_manager::get::<User>("USERS", user.as_str());
    match result {
        None => Err(reject::custom(UserFail::new(user))),
        Some(res) => Ok(warp::reply::json(&res)),
    }
}

pub async fn set_user_details(token: String, user: User) -> Result<impl Reply, Rejection> {
    let valid_jwt = auth::decode_token(token).await.or_else(|err| {
        log::error!("{:?}", err);
        Err(warp::reject())
    })?;

    let uid = auth::get_uid(valid_jwt).await.or_else(|err| {
        log::error!("{:?}", err);
        Err(warp::reject())
    })?;

    let result = redis_manager::set::<User>("USERS", &uid, user);
    match result {
        Some(value) => Ok(warp::reply::json(&value)),
        None => Err(reject::reject()),
    }
}

pub async fn get_user_from_token(token: String) -> Result<impl Reply, Rejection> {
    let valid_jwt = auth::decode_token(token).await.or_else(|err| {
        log::error!("{:?}", err);
        Err(warp::reject())
    })?;

    let uid = auth::get_uid(valid_jwt).await.or_else(|err| {
        log::error!("{:?}", err);
        Err(warp::reject())
    })?;

    get_user_details(uid).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_user_claims() {}
}
