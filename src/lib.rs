use std::convert::Infallible;
use std::net::SocketAddr;
use warp::Filter;
use vrp_pragmatic::format::problem::Problem;
use serde::{Deserialize, Serialize};


mod geocoding;
mod request;
mod vrp;

pub async fn start_server(addr: SocketAddr) {
    // TODO potentially move path parameterized geocoding to query
    let forward_geocoding =
        warp::path!("geocoding" / "forward" / String).and_then(receive_and_search_coordinates);

    let reverse_geocoding =
        warp::path!("geocoding" / "reverse" / f64 / f64).and_then(receive_and_search_postcode);

    let simple_trip =
        warp::path!("routing" / "solver" / "simple")
            .and(warp::post())
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json::<SimpleTrip>())
            .and_then(simple_trip);

    let simple_trip_async =
        warp::path!("routing" / "solver" / "simple" / "async")
            .and(warp::post())
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json::<SimpleTrip>())
            .and_then(simple_trip_async);

    let trip =
        warp::path!("routing" / "solver")
            // TODO fix compression .with(warp::compression::gzip())
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json())
            .and_then(trip);

    let routes = trip
        .or(simple_trip)
        .or(simple_trip_async)
        .or(forward_geocoding)
        .or(reverse_geocoding);

    warp::serve(routes).run(addr).await;
}

pub async fn receive_and_search_coordinates(postcode: String) -> Result<impl warp::Reply, Infallible> {
    let result = geocoding::search_coordinates(postcode.as_ref());
    Ok(result)
}

pub async fn receive_and_search_postcode(lat: f64, lon: f64) -> Result<impl warp::Reply, Infallible> {
    let result = geocoding::search_postcode(vec![lat, lon]);
    Ok(result)
}

pub async fn trip(request: Problem) -> Result<impl warp::Reply, Infallible> {
    // let result = geocoding::search_postcode(vec![lat, lon]);
    Ok("result")
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimpleTrip {
    pub coordinate_vehicles: Vec<String>,
    pub coordinate_jobs: Vec<String>
}

pub async fn simple_trip(trip: SimpleTrip) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&trip))
}

pub async fn simple_trip_async(trip: SimpleTrip) -> Result<impl warp::Reply, Infallible> {
    tokio::task::spawn(async {
        println!("Hey, i'm gonna be another task")
    });
    // let result = geocoding::search_postcode(vec![lat, lon]);
    Ok("result")
}
