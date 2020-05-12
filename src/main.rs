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
    let mut cache = LruCache::new(1);
    cache.put("postcodes", build_geocoding_csv());

    let postcodes = cache.get(&"postcodes").expect("Unable to get postcodes");
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
        .or(forward_geocodingkey.to_string().or(reverse_geocoding);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn build_geocoding_csv() -> Reader<File> {
    csv::Reader::from_path("postcodes.csv").expect("Issue reading postcodes.csv")
}
