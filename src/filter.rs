use crate::{geocoding, osrm_service, request, solver};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use vrp_pragmatic::checker::CheckerContext;
use vrp_pragmatic::format::problem::{Matrix, PragmaticProblem, Problem};

use warp::http::Method;

use crate::geocoding::{forward_search, reverse_search};
use crate::user::{get_id_from_token, get_user, set_user, User};
use serde::Serialize;
use warp::reply::Json;
use warp::{reject, Filter, Rejection};

pub async fn get_user_from_token(token: String) -> Result<impl warp::Reply, Rejection> {
    let user = get_user(token).await;
    match user {
        Ok(user) => {
            log::debug!("User: `{}`", user);
            Ok(warp::reply::json(&user))
        }
        Err(err) => {
            log::error!("Error getting user: `{}`", err);
            Err(warp::reject())
        }
    }
}
pub async fn set_user_from_token(token: String, user: User) -> Result<Json, Rejection> {
    let user_check = get_id_from_token(token).await;

    let result = set_user(user).await;
    match result {
        Some(value) => Ok(warp::reply::json(&value)),
        None => Err(reject::reject()),
    }
}

pub async fn search_coordinates(
    _token: String,
    postcode: String,
) -> Result<impl warp::Reply, Infallible> {
    // get_user_from_token(token).await.unwrap();
    let result = reverse_search(postcode);
    Ok(result)
}
pub async fn search_postcode(
    lat: f64,
    lon: f64,
    _token: String,
) -> Result<impl warp::Reply, Infallible> {
    // get_user_from_token(token).await.unwrap();
    let result = forward_search(vec![lat, lon]);
    Ok(result)
}

pub async fn simple_trip(
    token: String,
    trip: request::SimpleTrip,
) -> Result<impl warp::Reply, Rejection> {
    let user = get_user(token).await.unwrap();
    let user_reply = user.clone().wrap_reply().await?;
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

    let solution = &context.solution;
    let user = user.add_route(solution.clone());
    let route_set_result = set_user(user).await;

    match_option_to_warp(route_set_result, Some(&context.solution))
}

fn match_option_to_warp<T, R>(outer: Option<T>, real_value: Option<&R>) -> Result<Json, Rejection>
where
    T: Serialize,
    R: Serialize,
{
    if let Some(value) = real_value {
        match outer {
            Some(value) => Ok(warp::reply::json(&real_value)),
            None => Err(reject::reject()),
        }
    } else {
        match outer {
            Some(value) => Ok(warp::reply::json(&value)),
            None => Err(reject::reject()),
        }
    }
}

pub async fn simple_trip_matrix(
    token: String,
    trip: request::SimpleTrip,
) -> Result<impl warp::Reply, Rejection> {
    // get_user_from_token(token).await.unwrap();

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

pub async fn trip(token: String, _request: Problem) -> Result<impl warp::Reply, Infallible> {
    get_user_from_token(token).await.unwrap();
    Ok("result")
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