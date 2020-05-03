#![feature(option_result_contains)]

use std::borrow::BorrowMut;
use std::fs::File;
use std::sync::{Mutex, Arc};

use csv::Reader;
use warp::Filter;

use crate::geocoding::GeocodingKind::{COORDINATES, POSTCODE};

mod request;
mod vrp;
mod geocoding;

#[tokio::main]
async fn main() {
    let postcodes: Arc<Mutex<Reader<File>>> = Arc::new(build_geocoding_csv());
    let thread_postcodes = postcodes.clone();

    let geocoding = warp::get()
        .and(warp::path("geocoding"))
        .and(warp::query()
            .map(move |query: geocoding::Geocoding| {
                match query.query {
                    POSTCODE(postcode) => geocoding::search_coordinates(thread_postcodes.lock().unwrap().borrow_mut(), postcode.as_str()),
                    COORDINATES(lat_lon) => geocoding::search_postcode(thread_postcodes.lock().unwrap().borrow_mut(), lat_lon),
                }
            })
        )
        // .map(|res| warp::reply::json(res))
        ;

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
        .or(geocoding);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn build_geocoding_csv() -> Mutex<Reader<File>> {
    Mutex::new(csv::Reader::from_path("postcodes.csv")
        .expect("Issue reading postcodes.csv"))
}
