use grekko::start_server;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

async fn setup() {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3080);
    start_server(socket).await
}
