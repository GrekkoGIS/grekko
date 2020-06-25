#[macro_use]
extern crate cached;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use vrp_pragmatic::checker::CheckerContext;
use vrp_pragmatic::format::problem::{PragmaticProblem, Problem, Matrix};
use vrp_pragmatic::format::solution::Solution;
use warp::{Filter, reject, Rejection};
use warp::http::Method;

use crate::user::{User, UserFail};

mod geocoding;
mod redis_manager;
mod request;
mod solver;
mod user;
mod mapbox;

pub async fn start_server(addr: SocketAddr) {
    tokio::task::spawn(async {
        geocoding::get_postcodes();
    });

    let cors = warp::cors().allow_methods(&[Method::GET, Method::POST, Method::DELETE]);

    // TODO [#18]: potentially move path parameterized geocoding to query
    let forward_geocoding = warp::path!("geocoding" / "forward" / String)
        .and_then(receive_and_search_coordinates)
        .with(&cors);

    let reverse_geocoding = warp::path!("geocoding" / "reverse" / f64 / f64)
        .and_then(receive_and_search_postcode)
        .with(&cors);

    let user_geocoding = warp::path!("user" / String)
        .and_then(get_user_geocodings)
        .with(&cors);

    let create_user = warp::path!("user")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<user::User>())
        .and_then(set_user_geocodings)
        .with(&cors);

    let simple_trip = warp::path!("routing" / "solver" / "simple")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::SimpleTrip>())
        .and_then(simple_trip)
        .with(&cors);

    let simple_trip_matrix = warp::path!("routing" / "solver" / "simple" / "matrix")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::SimpleTrip>())
        .and_then(simple_trip_matrix)
        .with(&cors);

    let simple_trip_async = warp::path!("routing" / "solver" / "simple" / "async")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::SimpleTrip>())
        .and_then(simple_trip_async)
        .with(&cors);

    let trip = warp::path!("routing" / "solver")
        // TODO [#19]: fix compression .with(warp::compression::gzip())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and_then(trip)
        .with(&cors);

    let routes = trip
        .or(user_geocoding)
        .or(create_user)
        .or(simple_trip)
        .or(simple_trip_matrix)
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

pub async fn get_user_geocodings(user: String) -> Result<impl warp::Reply, Rejection> {
    let result = redis_manager::get::<user::User>("USERS", user.as_str());
    return match result {
        None => Err(reject::custom(UserFail::new())),
        Some(res) => Ok(warp::reply::json(&res)),
    };
}

pub async fn set_user_geocodings(user: User) -> Result<impl warp::Reply, Rejection> {
    let id = user.id.clone();
    let id = id.as_str();
    let result = redis_manager::set::<user::User>("USERS", id, user);
    return match result {
        Some(value) => Ok(warp::reply::json(&String::from(value))),
        None => Err(reject()),
    };
}

pub async fn trip(_request: Problem) -> Result<impl warp::Reply, Infallible> {
    // let result = geocoding::search_postcode(vec![lat, lon]);
    Ok("result")
}

pub async fn simple_trip(trip: request::SimpleTrip) -> Result<impl warp::Reply, Infallible> {
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

fn get_core_problem(problem: Problem, matrices: Option<Vec<Matrix>>) -> Arc<vrp_core::models::Problem> {
    Arc::new(
        if let Some(matrices) = matrices { (problem, matrices).read_pragmatic() } else { problem.read_pragmatic() }
            .ok()
            .unwrap(),
    )
}

pub async fn simple_trip_matrix(trip: request::SimpleTrip) -> Result<impl warp::Reply, Rejection> {
    if let Err(err) = apply_mapbox_max_jobs(&trip) {
        return Err(err)
    }

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

fn apply_mapbox_max_jobs(trip: &request::SimpleTrip) -> std::result::Result<(), warp::reject::Rejection> {
    if trip.coordinate_jobs.len() + trip.coordinate_vehicles.len() >= 25 {
        return Err(warp::reject::reject())
    } else {
        Ok(())
    }
}

async fn build_matrix(trip: &request::SimpleTrip) -> Matrix {
    let matrix_vehicles: Vec<Vec<f64>> = trip.clone()
        .coordinate_vehicles
        .iter()
        .map(|coordinate| geocoding::lookup_coordinates(String::from(coordinate)))
        .map(|location| vec![location.lng, location.lat])
        .collect();
    let matrix_jobs: Vec<Vec<f64>> = trip.clone()
        .coordinate_jobs
        .iter()
        .map(|coordinate| geocoding::lookup_coordinates(String::from(coordinate)))
        .map(|location| vec![location.lng, location.lat])
        .collect();
    let concat = [&matrix_jobs[..], &matrix_vehicles[..]].concat();

    let internal_matrix = mapbox::get_matrix(concat).await.unwrap_or_default();

    mapbox::convert_to_vrp_matrix(internal_matrix).await
}

pub async fn simple_trip_async(_trip: request::SimpleTrip) -> Result<impl warp::Reply, Infallible> {
    tokio::task::spawn(async { println!("Hey, i'm gonna be another task") });
    // let result = geocoding::search_postcode(vec![lat, lon]);
    Ok("result")
}
