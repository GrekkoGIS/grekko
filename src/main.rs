#![feature(option_result_contains)]
#[macro_use]
extern crate cached;

use std::borrow::BorrowMut;
use std::fs::File;
use std::sync::{Arc, Mutex};

use cached::SizedCache;
use csv::Reader;
use lru::LruCache;
use warp::Filter;
use tokio::sync::mpsc::channel;
use std::convert::Infallible;

mod request;
mod vrp;
mod geocoding;

// cached_key! {
//     CACHE: SizedCache<String, Reader<File>> = SizedCache::with_size(1);
//     Key = "Cache";
//     fn build_geocoding_csv() -> Reader<File> = {
//         let result = csv::Reader::from_path("postcodes.csv").expect("Issue reading postcodes.csv");
//         result.records().collect()
//     }
// }

#[tokio::main]
async fn main() {

    let forward_geocoding = warp::path!("geocoding" / "forward" / String)
        .and_then(receive_and_search_coordinates);

    let reverse_geocoding = warp::path!("geocoding" / "reverse" / f64 / f64)
        .and_then(receive_and_search_postcode);

    let trip = warp::post()
        .and(warp::path("detailed"))
        // .and(warp::path::param::<u32>())
        // .with(warp::compression::gzip())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .map(|request: request::DetailedRequest| {
            warp::reply::json(&request.convert_to_internal_problem())
        });

    let routes = trip
        .or(forward_geocoding).or(reverse_geocoding);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

async fn receive_and_search_coordinates(postcode: String) -> Result<impl warp::Reply, Infallible> {
    let result = geocoding::search_coordinates(postcode.as_ref());
    Ok(result)
}

async fn receive_and_search_postcode(lat: f64, lon: f64) -> Result<impl warp::Reply, Infallible> {
    let result = geocoding::search_postcode(vec![lat, lon]);
    Ok(result)
}
