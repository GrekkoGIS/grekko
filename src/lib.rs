use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use vrp_pragmatic::checker::CheckerContext;
use vrp_pragmatic::format::problem::{PragmaticProblem, Problem};
use vrp_pragmatic::format::solution::Solution;
use warp::Filter;

mod geocoding;
mod request;
mod solver;
mod redis_manager;

pub async fn start_server(addr: SocketAddr) {
    tokio::task::spawn(async {
        geocoding::bootstrap_cache(geocoding::POSTCODE_TABLE_NAME);
    });

    // TODO [#18]: potentially move path parameterized geocoding to query
    let forward_geocoding =
        warp::path!("geocoding" / "forward" / String).and_then(receive_and_search_coordinates);

    let reverse_geocoding =
        warp::path!("geocoding" / "reverse" / f64 / f64).and_then(receive_and_search_postcode);

    let simple_trip = warp::path!("routing" / "solver" / "simple")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::SimpleTrip>())
        .and_then(simple_trip);

    let simple_trip_async = warp::path!("routing" / "solver" / "simple" / "async")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::SimpleTrip>())
        .and_then(simple_trip_async);

    let trip = warp::path!("routing" / "solver")
        // TODO [#19]: fix compression .with(warp::compression::gzip())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and_then(trip);

    let routes = trip
        .or(simple_trip)
        .or(simple_trip_async)
        .or(forward_geocoding)
        .or(reverse_geocoding);

    println!("Server is starting on {}", addr);
    warp::serve(routes).run(addr).await;
}

pub async fn receive_and_search_coordinates(
    postcode: String,
) -> Result<impl warp::Reply, Infallible> {
    let result = geocoding::reverse_search(postcode);
    Ok(result)
}

pub async fn receive_and_search_postcode(
    lat: f64,
    lon: f64,
) -> Result<impl warp::Reply, Infallible> {
    let result = geocoding::forward_search(vec![lat, lon]);
    Ok(result)
}

pub async fn trip(request: Problem) -> Result<impl warp::Reply, Infallible> {
    // let result = geocoding::search_postcode(vec![lat, lon]);
    Ok("result")
}

pub async fn simple_trip(trip: request::SimpleTrip) -> Result<impl warp::Reply, Infallible> {
    // Convert simple trip to internal problem
    let problem = trip.clone().convert_to_internal_problem().await;
    // Convert internal problem to a core problem
    let core_problem = problem.read_pragmatic();
    // Create an ARC for it
    let problem =
        Arc::new(core_problem.expect("Could not read a pragmatic problem into a core problem"));
    // Start building a solution
    let (solution, _) = solver::create_builder(&problem);
    // Convert that to a pragmatic solution
    let solution: Solution =
        solver::get_pragmatic_solution(&Arc::try_unwrap(problem).ok().unwrap(), &solution);

    // TODO [#20]: this context builder is silly, refactor it
    let problem: Problem = trip.convert_to_internal_problem().await;
    let context = CheckerContext::new(problem, None, solution);

    if let Err(err) = context.check() {
        format!("unfeasible solution in '{}': '{}'", "name", err);
    }

    Ok(warp::reply::json(&context.solution))
}

pub async fn simple_trip_async(trip: request::SimpleTrip) -> Result<impl warp::Reply, Infallible> {
    tokio::task::spawn(async { println!("Hey, i'm gonna be another task") });
    // let result = geocoding::search_postcode(vec![lat, lon]);
    Ok("result")
}
