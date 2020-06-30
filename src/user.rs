use serde::export::fmt;
use serde::{Deserialize, Serialize};
use warp::{reject, Reply, Rejection};
use warp::reply::Response;
use jsonwebtoken::{dangerous_unsafe_decode, Validation, decode, TokenData, DecodingKey, Algorithm, decode_header};
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
    decode_token(user.clone());
    let result = redis_manager::get::<User>("USERS", user.as_str());
    match result {
        None => Err(reject::custom(UserFail::new(user))),
        Some(res) => Ok(warp::reply::json(&res)),
    }
}

pub async fn set_user_details(token: String, user: User) -> Result<impl warp::Reply, Rejection> {
    decode_token(token);
    let id = user.id.clone();
    let id = id.as_str();
    let result = redis_manager::set::<User>("USERS", id, user);
    match result {
        Some(value) => Ok(warp::reply::json(&value)),
        None => Err(reject()),
    }
}

pub async fn get_user_claims(
    token: String,
) -> Result<impl Reply, Rejection> {
    let uid = decode_token(token).claims.uid;
    get_user_details(uid).await
}

fn decode_token(token: String) -> TokenData<Claims> {
    let token: Vec<&str> = token.split("Bearer ").collect();
    let token = token.get(1).unwrap().clone();
    let mut validation = Validation::default();

    validation.set_audience(&["api://default"]);
    validation.leeway = 60;
    validation.iss = Some(String::from("https://dev-201460.okta.com/oauth2/default"));
    validation.validate_exp = true;
    validation.algorithms = vec![Algorithm::RS256, Algorithm::HS256, Algorithm::RS512];

    log::debug!("Validation: {:?}", validation);

    let header = decode_header(&token);
    log::debug!("Token header: {:?}", header);
    decode::<Claims>(&token, &DecodingKey::from_secret("".as_ref()), &validation).unwrap()
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
