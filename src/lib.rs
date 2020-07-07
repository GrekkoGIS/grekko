#[macro_use]
extern crate cached;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use vrp_pragmatic::checker::CheckerContext;
use vrp_pragmatic::format::problem::{Matrix, PragmaticProblem, Problem};

use warp::http::Method;

use warp::{Filter, Rejection};

use crate::user::get_user_from_token;

pub mod auth;
pub mod geocoding;
mod mapbox;
mod osrm_service;
mod redis_manager;
mod request;
mod solver;
pub mod user;

pub async fn start_server(addr: SocketAddr) {
    tokio::task::spawn(async {
        geocoding::get_postcodes();
    });

    const AUTH_HEADER: &str = "authorization";

    let cors = warp::cors()
        .allow_methods(&[Method::GET, Method::POST])
        .allow_header(AUTH_HEADER);

    let user_extractor = warp::path("user")
        .and(warp::get())
        .and(warp::header::<String>(AUTH_HEADER))
        .and_then(user::get_user_from_token);

    let create_user = warp::path!("user" / "create")
        .and(warp::header::<String>(AUTH_HEADER))
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<user::User>())
        .and_then(user::set_user_details);

    // TODO [#18]: potentially move path parameterized geocoding to query
    let forward_geocoding = warp::path!("geocoding" / "forward" / String)
        .and(warp::header::<String>(AUTH_HEADER))
        .and_then(geocoding::receive_and_search_coordinates);

    let reverse_geocoding = warp::path!("geocoding" / "reverse" / f64 / f64)
        .and(warp::header::<String>(AUTH_HEADER))
        .and_then(geocoding::receive_and_search_postcode);

    let simple_trip = warp::path!("routing" / "solver" / "simple")
        .and(warp::header::<String>(AUTH_HEADER))
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::SimpleTrip>())
        .and_then(simple_trip);

    let simple_trip_matrix = warp::path!("routing" / "solver" / "simple" / "matrix")
        .and(warp::header::<String>(AUTH_HEADER))
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::SimpleTrip>())
        .and_then(simple_trip_matrix);

    let simple_trip_async = warp::path!("routing" / "solver" / "simple" / "async")
        .and(warp::header::<String>(AUTH_HEADER))
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::SimpleTrip>())
        .and_then(simple_trip_async);

    let trip = warp::path!("routing" / "solver")
        .and(warp::header::<String>(AUTH_HEADER))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and_then(trip);

    let routes = trip
        .or(user_extractor)
        .or(create_user)
        .or(simple_trip)
        .or(simple_trip_matrix)
        .or(simple_trip_async)
        .or(forward_geocoding)
        .or(reverse_geocoding)
        // TODO [#19]: fix compression .with(warp::compression::gzip())
        // .with(warp::compression::gzip())
        .with(&cors);

    log::info!("Server is starting on {}", addr);
    warp::serve(routes).run(addr).await;
}

pub async fn trip(token: String, _request: Problem) -> Result<impl warp::Reply, Infallible> {
    get_user_from_token(token).await.unwrap();
    Ok("result")
}

pub async fn simple_trip(
    token: String,
    trip: request::SimpleTrip,
) -> Result<impl warp::Reply, Infallible> {
    get_user_from_token(token).await.unwrap();
    // TODO [#29]: add some concurrency here
    // Convert simple trip to internal problem
    let problem = trip.clone().convert_to_internal_problem().await;
    // Convert internal problem to a core problem
    let core_problem = problem.read_pragmatic();
    // Create an ARC for it
    let problem =
        Arc::new(core_problem.expect("Could not read a pragmatic problem into a core problem"));
    // Start building a solution
    let (solution, _, _) = solver::solve_problem(solver::create_solver(problem.clone()));
    // Convert that to a pragmatic solution
    let (solution, _) =
        solver::get_pragmatic_solution(&Arc::try_unwrap(problem).ok().unwrap(), &solution);

    // TODO [#20]: this context builder is silly, refactor it
    let problem: Problem = trip.convert_to_internal_problem().await;
    let context = CheckerContext::new(problem, None, solution);

    if let Err(err) = context.check() {
        format!("unfeasible solution in '{}': '{}'", "name", err);
    }

    Ok(warp::reply::json(&context.solution))
}

fn get_core_problem(
    problem: Problem,
    matrices: Option<Vec<Matrix>>,
) -> Arc<vrp_core::models::Problem> {
    Arc::new(
        if let Some(matrices) = matrices {
            (problem, matrices).read_pragmatic()
        } else {
            problem.read_pragmatic()
        }
        .ok()
        .unwrap(),
    )
}

pub async fn simple_trip_matrix(
    token: String,
    trip: request::SimpleTrip,
) -> Result<impl warp::Reply, Rejection> {
    get_user_from_token(token).await.unwrap();

    let problem = trip.clone().convert_to_internal_problem().await;

    let matrix = build_matrix(&trip).await;

    let matrix_copy = matrix.clone();

    let problem = get_core_problem(problem, Some(vec![matrix]));

    let (solution, _, _) = solver::solve_problem(solver::create_solver(problem.clone()));

    let (solution, _) =
        solver::get_pragmatic_solution(&Arc::try_unwrap(problem).ok().unwrap(), &solution);

    let problem: Problem = trip.convert_to_internal_problem().await;

    let context = CheckerContext::new(problem, Some(vec![matrix_copy]), solution);
    if let Err(err) = context.check() {
        format!("unfeasible solution in '{}': '{}'", "name", err);
    }

    Ok(warp::reply::json(&context.solution))
}

fn apply_mapbox_max_jobs(trip: &request::SimpleTrip) -> std::result::Result<(), Rejection> {
    if trip.coordinate_jobs.len() + trip.coordinate_vehicles.len() >= 25 {
        Err(warp::reject::reject())
    } else {
        Ok(())
    }
}

async fn build_matrix(trip: &request::SimpleTrip) -> Matrix {
    let matrix_vehicles: Vec<Vec<f32>> = trip
        .clone()
        .coordinate_vehicles
        .iter()
        .map(|coordinate| geocoding::lookup_coordinates(String::from(coordinate)))
        .map(|location| vec![location.lng as f32, location.lat as f32])
        .collect();
    let matrix_jobs: Vec<Vec<f32>> = trip
        .clone()
        .coordinate_jobs
        .iter()
        .map(|coordinate| geocoding::lookup_coordinates(String::from(coordinate)))
        .map(|location| vec![location.lng as f32, location.lat as f32])
        .collect();
    let concat = [&matrix_jobs[..], &matrix_vehicles[..]].concat();

    osrm_service::get_matrix(concat).unwrap()
}

pub async fn simple_trip_async(
    token: String,
    _trip: request::SimpleTrip,
) -> Result<impl warp::Reply, Rejection> {
    get_user_from_token(token).await.unwrap();
    tokio::task::spawn(async { println!("Hey, i'm gonna be another task") });
    // let result = geocoding::search_postcode(vec![lat, lon]);
    Ok("result")
}
