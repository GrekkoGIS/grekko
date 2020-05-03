#![feature(option_result_contains)]

use std::borrow::BorrowMut;
use std::fs::File;
use std::sync::{Arc, Mutex};

use csv::Reader;
use warp::Filter;

use crate::geocoding::GeocodingKind::{COORDINATES, POSTCODE};

mod request;
mod vrp;
mod geocoding;

#[tokio::main]
async fn main() {
    let postcodes: Arc<Mutex<Reader<File>>> = Arc::new(build_geocoding_csv());
    let forward_geocoding_postcodes = postcodes.clone();
    let reverse_geocoding_postcodes = postcodes.clone();

    let forward_geocoding = warp::path!("geocoding" / "forward" / String)
        .map(move |postcode: String| {
            geocoding::search_coordinates(forward_geocoding_postcodes
                                              .lock()
                                              .expect("Mutex was poisoned")
                                              .borrow_mut(), postcode.as_str())
        });

    let reverse_geocoding = warp::path!("geocoding" / "reverse" / f64 / f64)
        .map(move |lat, lon| {
            geocoding::search_postcode(reverse_geocoding_postcodes
                                           .lock()
                                           .expect("Mutex was poisoned")
                                           .borrow_mut(), vec![lat, lon])
        });

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
        .or(forward_geocoding)
        .or(reverse_geocoding);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn build_geocoding_csv() -> Mutex<Reader<File>> {
    Mutex::new(csv::Reader::from_path("postcodes.csv")
        .expect("Issue reading postcodes.csv"))
}
