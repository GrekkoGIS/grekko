use serde::export::fmt;
use serde::{Deserialize, Serialize};
use vrp_pragmatic::format::solution::Solution;
use warp::reply::{Json, Response};
use warp::{reject, Rejection};

use crate::user::get_user_details;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub(crate) uid: String,
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
            Err(_) => Err(reject::reject()), //TODO
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
