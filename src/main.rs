#![feature(option_result_contains)]
#[macro_use]
extern crate cached;

use warp::Filter;
use std::convert::Infallible;
use grekko::start_server;
use std::net::{SocketAddrV4, Ipv4Addr, IpAddr, SocketAddr};

pub mod request;
mod vrp;
pub mod geocoding;

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
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3030);
    start_server(socket).await;
}
