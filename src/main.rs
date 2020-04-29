use std::io::{BufReader, BufWriter};
use std::sync::Arc;

use vrp_core::models::{Problem as CoreProblem, Solution as CoreSolution};
use vrp_core::solver::Builder;
use vrp_pragmatic::checker::CheckerContext;
use vrp_pragmatic::format::problem::{deserialize_problem, PragmaticProblem, Problem};
use vrp_pragmatic::format::solution::{deserialize_solution, PragmaticSolution, Solution};

fn main() {
    println!("Hello, world!");
    test_pragmatic();
}

fn test_pragmatic() {
    let problem_text = r#"
{
  "plan": {
    "jobs": [
      {
        "id": "multi_job1",
        "pickups": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5622847,
                  "lng": 13.4023099
                },
                "duration": 240.0
              }
            ],
            "demand": [
              1
            ],
            "tag": "p1"
          },
          {
            "places": [
              {
                "location": {
                  "lat": 52.5330881,
                  "lng": 13.3973059
                },
                "duration": 240.0
              }
            ],
            "demand": [
              1
            ],
            "tag": "p2"
          }
        ],
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5252832,
                  "lng": 13.4188422
                },
                "duration": 240.0
              }
            ],
            "demand": [
              2
            ],
            "tag": "d1"
          }
        ]
      },
      {
        "id": "multi_job2",
        "pickups": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.52599,
                  "lng": 13.45413
                },
                "duration": 240.0
              }
            ],
            "demand": [
              2
            ],
            "tag": "p1"
          }
        ],
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4928,
                  "lng": 13.4597
                },
                "duration": 240.0
              }
            ],
            "demand": [
              1
            ],
            "tag": "d1"
          },
          {
            "places": [
              {
                "location": {
                  "lat": 52.4989,
                  "lng": 13.3917
                },
                "duration": 240.0
              }
            ],
            "demand": [
              1
            ],
            "tag": "d2"
          }
        ]
      }
    ]
  },
  "fleet": {
    "vehicles": [
      {
        "typeId": "vehicle",
        "vehicleIds": [
          "vehicle_1"
        ],
        "profile": "normal_car",
        "costs": {
          "fixed": 22.0,
          "distance": 0.0002,
          "time": 0.004806
        },
        "shifts": [
          {
            "start": {
              "time": "2019-07-04T09:00:00Z",
              "location": {
                "lat": 52.4664257,
                "lng": 13.2812488
              }
            },
            "end": {
              "time": "2019-07-04T18:00:00Z",
              "location": {
                "lat": 52.4664257,
                "lng": 13.2812488
              }
            }
          }
        ],
        "capacity": [
          10
        ]
      }
    ],
    "profiles": [
      {
        "name": "normal_car",
        "type": "car"
      }
    ]
  }
}
    "#;

    let problem = String::from(problem_text).read_pragmatic();

    let problem = Arc::new(problem.expect("Problem could not be marshalled to an arc"));
    let (solution, _) = Builder::default()
        .with_max_generations(Some(10))
        .with_problem(problem.clone())
        .build()
        .unwrap_or_else(|err| panic!("cannot build solver: {}", err))
        .solve()
        .unwrap_or_else(|err| panic!("cannot solver problem: {}", err));

    let solution = get_pragmatic_solution(&Arc::try_unwrap(problem).ok().unwrap(), &solution);
    let problem = get_pragmatic_problem(problem_text);

    // TODO use matrices

    let context = CheckerContext::new(problem, None, solution);

    if let Err(err) = context.check() {
        panic!("unfeasible solution in '{}': '{}'", "name", err);
    }

    println!("{:?}", context.solution)
}

fn get_pragmatic_problem(problem_text: &str) -> Problem {
    deserialize_problem(BufReader::new(problem_text.as_bytes())).unwrap()
}

fn get_pragmatic_solution(problem: &CoreProblem, solution: &CoreSolution) -> Solution {
    let mut buffer = String::new();
    let writer = unsafe { BufWriter::new(buffer.as_mut_vec()) };

    solution
        .write_pragmatic_json(&problem, writer)
        .expect("cannot write pragmatic solution");

    deserialize_solution(BufReader::new(buffer.as_bytes())).expect("cannot deserialize solution")
}
