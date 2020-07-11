#![feature(option_result_contains)]
#![feature(in_band_lifetimes)]

#[macro_use]
extern crate cached;
#[macro_use]
extern crate log;

use grekko::start_server;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub mod auth;
pub mod filter;
pub mod geocoding;
pub mod osrm_service;
pub mod redis_manager;
pub mod request;
pub mod solver;
pub mod user;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let port = option_env!("SERVER_PORT")
        .or_else(|| Some("3030"))
        .unwrap()
        .parse::<u16>()
        .unwrap();
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    start_server(socket).await;
}
