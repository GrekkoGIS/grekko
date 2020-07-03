use std::io::{BufRead, BufReader, BufWriter};
use std::sync::Arc;

use vrp_core::models::{Problem as CoreProblem, Solution as CoreSolution};
use vrp_core::solver::{Builder, Metrics, Solver};
use vrp_pragmatic::format::problem::{deserialize_problem, Problem};
use vrp_pragmatic::format::solution::{deserialize_solution, PragmaticSolution, Solution};

pub fn get_pragmatic_problem(problem_text: &str) -> Problem {
    deserialize_problem(BufReader::new(problem_text.as_bytes())).unwrap()
}

pub fn get_pragmatic_solution(problem: &CoreProblem, solution: &CoreSolution) -> (Solution, String) {
    (build_pragmatic_solution(&problem, &solution), build_geo_json(&problem, &solution))
}

pub fn build_pragmatic_solution(problem: &CoreProblem, solution: &CoreSolution) -> Solution {
    let mut buffer = String::new();
    // TODO [#36]: don't be unsafe
    let writer = unsafe { BufWriter::new(buffer.as_mut_vec()) };
    solution
        .write_pragmatic_json(&problem, writer)
        .expect("Unable to write solution");

    deserialize_solution(BufReader::new(buffer.as_bytes())).expect("cannot deserialize solution")
}

pub fn build_geo_json(problem: &CoreProblem, solution: &CoreSolution) -> String {
    // TODO [#36]: don't be unsafe
    let mut buffer_geojson = String::new();
    let writer = unsafe { BufWriter::new(buffer_geojson.as_mut_vec()) };
    solution
        .write_geo_json(&problem, writer)
        .expect("Unable to write geojson");

    BufReader::new(buffer_geojson.as_bytes()).lines().map(|l| l.unwrap()).collect()
}

pub fn create_solver(problem: Arc<CoreProblem>) -> Solver {
    Builder::new(problem)
        .with_max_generations(Some(100))
        .with_max_time(Some(90))
        .build()
        .unwrap_or_else(|err| panic!("cannot build solver, error: {}", err))
}

pub fn solve_problem(solver: Solver) -> (CoreSolution, f64, Option<Metrics>) {
    solver
        .solve()
        .unwrap_or_else(|err| panic!("cannot solve problem, error: {}", err))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use vrp_pragmatic::checker::CheckerContext;
    use vrp_pragmatic::format::problem::{PragmaticProblem, Problem};
    use vrp_pragmatic::format::solution::Solution;

    use crate::solver;
    use crate::solver::{
        create_solver, get_pragmatic_problem, get_pragmatic_solution, solve_problem,
    };

    #[test]
    fn test_pragmatic() {
        let problem_text = r#"
{
  "plan": {
    "jobs": [
      {
        "id": "job1",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5697304,
                  "lng": 13.3848221
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job2",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5060419,
                  "lng": 13.5152641
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job3",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5421315,
                  "lng": 13.5189513
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job4",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5243421,
                  "lng": 13.4619776
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job5",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4629002,
                  "lng": 13.4757055
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job6",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4960479,
                  "lng": 13.3915876
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job7",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5372914,
                  "lng": 13.3996298
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job8",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5429597,
                  "lng": 13.3989552
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job9",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5678751,
                  "lng": 13.4231417
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job10",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4945572,
                  "lng": 13.4698049
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job11",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4989511,
                  "lng": 13.4740528
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job12",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4658835,
                  "lng": 13.4461224
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job13",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5685168,
                  "lng": 13.3690720
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job14",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4742821,
                  "lng": 13.3628588
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job15",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5650163,
                  "lng": 13.3027992
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job16",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5496702,
                  "lng": 13.4286263
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job17",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5058684,
                  "lng": 13.4750990
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job18",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5473416,
                  "lng": 13.3327894
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job19",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5276784,
                  "lng": 13.5465640
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job20",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5192039,
                  "lng": 13.3044440
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job21",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5228904,
                  "lng": 13.4418623
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job22",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4828453,
                  "lng": 13.4363713
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job23",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5291335,
                  "lng": 13.3668934
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job24",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5261554,
                  "lng": 13.5062954
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job25",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5189653,
                  "lng": 13.3890068
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job26",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5090143,
                  "lng": 13.4368189
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job27",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4940454,
                  "lng": 13.3788834
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job28",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5065998,
                  "lng": 13.3689955
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job29",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5473490,
                  "lng": 13.3733163
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job30",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4695374,
                  "lng": 13.4914662
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job31",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4868236,
                  "lng": 13.3353656
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job32",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4661617,
                  "lng": 13.3226920
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job33",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4917198,
                  "lng": 13.5251532
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job34",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5431264,
                  "lng": 13.4416407
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job35",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5426716,
                  "lng": 13.5161692
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job36",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4708241,
                  "lng": 13.3598752
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job37",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4737341,
                  "lng": 13.3866700
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job38",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5404107,
                  "lng": 13.3914127
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job39",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5492619,
                  "lng": 13.3693560
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job40",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4827319,
                  "lng": 13.3157235
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job41",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4711004,
                  "lng": 13.3321906
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job42",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4871049,
                  "lng": 13.5423247
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job43",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5614441,
                  "lng": 13.4194712
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job44",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5414557,
                  "lng": 13.5276390
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job45",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5425207,
                  "lng": 13.4139155
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job46",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5632095,
                  "lng": 13.2940051
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job47",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5146285,
                  "lng": 13.2852959
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job48",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4855438,
                  "lng": 13.3832067
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job49",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.5279215,
                  "lng": 13.4995315
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
          }
        ]
      },
      {
        "id": "job50",
        "deliveries": [
          {
            "places": [
              {
                "location": {
                  "lat": 52.4959052,
                  "lng": 13.3539713
                },
                "duration": 180.0
              }
            ],
            "demand": [
              1
            ]
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
          "vehicle_1",
          "vehicle_2",
          "vehicle_3",
          "vehicle_4",
          "vehicle_5"
        ],
        "profile": "car",
        "costs": {
          "fixed": 20.0,
          "distance": 0.0002,
          "time": 0.005
        },
        "shifts": [
          {
            "start": {
              "time": "1970-01-01T00:00:00Z",
              "location": {
                "lat": 52.4664257,
                "lng": 13.2812488
              }
            },
            "end": {
              "time": "1970-01-01T23:59:00Z",
              "location": {
                "lat": 52.4664257,
                "lng": 13.2812488
              }
            }
          }
        ],
        "capacity": [
          20
        ]
      }
    ],
    "profiles": [
      {
        "name": "car",
        "type": "car"
      }
    ]
  },
  "objectives": {
    "primary": [
      {
        "type": "minimize-unassigned"
      }
    ],
    "secondary": [
      {
        "type": "balance-distance"
      },
      {
        "type": "minimize-cost"
      }
    ]
  }
}
    "#;

        let problem = String::from(problem_text).read_pragmatic();
        let problem = Arc::new(problem.expect("Problem could not be marshalled to an arc"));
        let (solution, _, _) = solve_problem(create_solver(problem.clone()));

        let (solution, _) =
            solver::get_pragmatic_solution(&Arc::try_unwrap(problem).ok().unwrap(), &solution);
        let problem: Problem = get_pragmatic_problem(problem_text);

        // TODO [#26]: use matrices potentially

        let context = CheckerContext::new(problem, None, solution);

        if let Err(err) = context.check() {
            panic!("unfeasible solution in '{}': '{}'", "name", err);
        }

        println!("{:?}", context.solution);
        assert!(true)
    }
}
