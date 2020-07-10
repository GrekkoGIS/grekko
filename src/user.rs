use crate::auth;
use crate::redis_manager;
use failure::{Error, ResultExt};
use serde::export::fmt;
use serde::{Deserialize, Serialize};
use vrp_pragmatic::format::solution::Solution;
use warp::reply::{Json, Response};
use warp::{reject, Rejection, Reply};

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
            Err(res) => Err(reject::custom(UserFail::new(String::new()))), //TODO
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

pub async fn get_user_filter(token: String) -> Result<impl warp::Reply, Rejection> {
    let user = get_user_from_token(token).await;
    match user {
        Ok(user) => {
            log::debug!("User: `{}`", user);
            Ok(warp::reply::json(&user))
        }
        Err(err) => {
            log::error!("Error getting user: `{}`", err);
            Err(warp::reject())
        }
    }
}

pub async fn get_user_from_token(token: String) -> Result<User, Error> {
    let uid = get_id_from_token(token).await?;
    log::debug!("User id decoded from token: `{}`", uid);

    get_user_details(uid).await
}

pub async fn get_user_details(uid: String) -> Result<User, Error> {
    redis_manager::get::<User>("USERS", uid.as_str())
}

pub async fn set_user(user: User) -> Option<String> {
    redis_manager::set::<User>("USERS", &user.uid, user.clone())
}

pub async fn set_user_details(token: String, user: User) -> Result<Json, Rejection> {
    let user_check = get_id_from_token(token).await;

    let result = set_user(user).await;
    match result {
        Some(value) => Ok(warp::reply::json(&value)),
        None => Err(reject::reject()),
    }
}

pub async fn get_id_from_token(token: String) -> Result<String, Error> {
    let valid_jwt = auth::decode_token(token).await?;

    let uid = auth::get_uid(valid_jwt).await?;

    Ok(uid)
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_get_user_claims() {}
}
