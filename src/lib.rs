use std::convert::Infallible;
use std::net::SocketAddr;
use warp::Filter;

mod geocoding;
mod request;
mod vrp;

pub async fn start_server(addr: SocketAddr) {
    // TODO potentially move path parameterized geocoding to query
    let forward_geocoding =
        warp::path!("geocoding" / "forward" / String).and_then(receive_and_search_coordinates);

    let reverse_geocoding =
        warp::path!("geocoding" / "reverse" / f64 / f64).and_then(receive_and_search_postcode);

    let trip = warp::post()
        .and(warp::path("detailed"))
        // .and(warp::path::param::<u32>())
        // TODO fix compression .with(warp::compression::gzip())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .map(|request: request::DetailedRequest| {
            warp::reply::json(&request.convert_to_internal_problem())
        });

    let routes = trip.or(forward_geocoding).or(reverse_geocoding);

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
