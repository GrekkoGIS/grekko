#![feature(const_fn)]

#[macro_use]
extern crate cached;

use std::net::SocketAddr;

use vrp_pragmatic::format::problem::{Matrix, PragmaticProblem, Problem};

use warp::http::Method;

use warp::{reject, Filter, Rejection};

use serde::de::DeserializeOwned;
use serde::Serialize;
use warp::reply::Json;

pub mod auth;
pub mod filter;
pub mod geocoding;
mod osrm_service;
mod redis_manager;
mod request;
mod solver;
pub mod user;

pub async fn start_server(address: SocketAddr) {
    tokio::task::spawn(async {
        geocoding::is_bootstrapped();
    });

    const AUTH_HEADER: &str = "authorization";

    let cors = warp::cors()
        .allow_methods(&[Method::GET, Method::POST])
        .allow_header(AUTH_HEADER);

    let user_extractor = warp::path("user")
        .and(warp::get())
        .and(warp::header::<String>(AUTH_HEADER))
        .and_then(filter::get_user_from_token);

    let create_user = warp::path!("user" / "create")
        .and(warp::header::<String>(AUTH_HEADER))
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 2))
        .and(warp::body::json::<user::structs::User>())
        .and_then(filter::set_user_from_token);

    // TODO [#18]: potentially move path parameterized geocoding to query
    let forward_geocoding = warp::path!("geocoding" / "forward" / String)
        .and(warp::header::<String>(AUTH_HEADER))
        .and_then(filter::search_coordinates);

    let reverse_geocoding = warp::path!("geocoding" / "reverse" / f64 / f64)
        .and(warp::header::<String>(AUTH_HEADER))
        .and_then(filter::search_postcode);

    let simple_trip = warp::path!("routing" / "solver" / "simple")
        .and(warp::header::<String>(AUTH_HEADER))
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 24))
        .and(warp::body::json::<request::SimpleTrip>())
        .and_then(filter::simple_trip);

    let simple_trip_matrix = warp::path!("routing" / "solver" / "simple" / "matrix")
        .and(warp::header::<String>(AUTH_HEADER))
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 24))
        .and(warp::body::json::<request::SimpleTrip>())
        .and_then(filter::simple_trip_matrix);

    let simple_trip_async = warp::path!("routing" / "solver" / "simple" / "async")
        .and(warp::header::<String>(AUTH_HEADER))
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 24))
        .and(warp::body::json::<request::SimpleTrip>())
        .and_then(filter::simple_trip_async);

    let trip = warp::path!("routing" / "solver")
        .and(warp::header::<String>(AUTH_HEADER))
        .and(warp::body::content_length_limit(1024 * 24))
        .and(warp::body::json())
        .and_then(filter::trip);

    let routes = trip
        .or(user_extractor)
        .or(create_user)
        .or(simple_trip)
        .or(simple_trip_matrix)
        .or(simple_trip_async)
        .or(forward_geocoding)
        .or(reverse_geocoding)
        // TODO [#19]: fix compression .with(warp::compression::gzip())
        // .with(warp::compression::gzip())
        .with(&cors);

    log::info!("Server is starting on {}", address);
    warp::serve(routes).run(address).await;
}
