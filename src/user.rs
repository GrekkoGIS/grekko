use warp::reject;
use warp::reply::Response;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    id: String,
    forward_geocoding: Vec<String>,
    reverse_geocoding: Vec<f64>,
    simple_routes: Option<Vec<String>>,
}

impl warp::reply::Reply for User {
    fn into_response(self) -> Response {
        Response::new(serde_json::to_string(&self).expect("Unable to serialise User").into())
    }
}

#[derive(Debug)]
pub struct UserFail {
    message: String
}

impl reject::Reject for UserFail {}

impl UserFail {
    pub fn new() -> UserFail {
        UserFail { message: String::from("Unable to find a user with that id") }
    }
}