use warp::reject;
use warp::reply::Response;
use serde::{Deserialize, Serialize};
use serde::export::fmt;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,
    forward_geocoding: Vec<String>,
    reverse_geocoding: Vec<Vec<f64>>,
    simple_routes: Vec<String>
}

impl warp::reply::Reply for User {
    fn into_response(self) -> Response {
        Response::new(serde_json::to_string(&self).expect("Unable to serialise User").into())
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ID: {} ForwardGeocoding: {:?}, ReverseGeocoding: {:?}, SimpleRoutes: {:?}",
               self.id, self.forward_geocoding, self.reverse_geocoding, self.simple_routes)
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