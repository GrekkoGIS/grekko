#![feature(option_result_contains)]

use std::fs::File;
use std::sync::{Arc, Mutex};

use csv::Reader;
use warp::Filter;
use std::borrow::BorrowMut;

mod request;
mod vrp;
mod geocoding;

#[tokio::main]
async fn main() {
    let postcodes: Mutex<Reader<File>> = build_geocoding_csv();

    println!("{:?}", geocoding::search(postcodes.lock().unwrap().borrow_mut(), "BS1 1AA"));

    let trip = warp::post()
        .and(warp::path("detailed"))
        // .and(warp::path::param::<u32>())
        // .with(warp::compression::gzip())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .map(|request: request::DetailedRequest| {
            warp::reply::json(&request)
        });

    let routes = trip
        .or(trip);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn build_geocoding_csv() -> Mutex<Reader<File>> {
    Mutex::new(csv::Reader::from_path("postcodes.csv")
        .expect("Issue reading postcodes.csv"))
}
