use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use vrp_pragmatic::format::{Location, problem};
use vrp_pragmatic::format::problem::{
    JobPlace, JobTask, Profile, VehicleCosts, VehiclePlace, VehicleShift, VehicleType,
};
use vrp_pragmatic::format::problem::Fleet as ProblemFleet;
use vrp_pragmatic::format::problem::Job as ProblemJob;
use vrp_pragmatic::format::problem::Plan as ProblemPlan;

use crate::geocoding;
use chrono::Duration;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailedRequest {
    pub plan: Plan,
    pub fleet: Fleet,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub jobs: Vec<Job>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub deliveries: Vec<Delivery>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Delivery {
    pub places: Vec<Place>,
    pub priority: i64,
    pub properties: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub postcode: String,
    pub duration: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fleet {
    pub vehicles: Vec<Vehicle>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vehicle {
    pub vehicle_ids: Vec<String>,
    pub costs: Costs,
    pub shifts: Vec<Shift>,
    pub capacity: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Costs {
    pub fixed: f64,
    pub distance: f64,
    pub time: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shift {
    pub start: Start,
    pub end: End,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Start {
    pub time: String,
    pub postcode: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct End {
    pub time: String,
    pub postcode: String,
}

impl DetailedRequest {
    pub fn convert_to_internal_problem(self) -> vrp_pragmatic::format::problem::Problem {
        let fleet = self
            .fleet
            .vehicles
            .iter()
            .map(|vehicle| {
                vrp_pragmatic::format::problem::VehicleType {
                    type_id: "vehicle".to_owned(), // TODO: understand type id's in Vehicle Type
                    vehicle_ids: (*vehicle.vehicle_ids).to_owned(),
                    profile: "car".to_string(), // TODO: enumerate the profile for the simple problem
                    costs: vrp_pragmatic::format::problem::VehicleCosts {
                        fixed: Option::from(vehicle.costs.fixed),
                        distance: vehicle.costs.distance,
                        time: vehicle.costs.time,
                    },
                    shifts: vehicle
                        .shifts
                        .iter()
                        .map(|shift| VehicleShift {
                            start: VehiclePlace {
                                time: shift.start.time.to_string(),
                                // TODO: utilise geocoding to get coordinates
                                location: Location { lat: 0.0, lng: 0.0 },
                            },
                            end: Option::from(VehiclePlace {
                                time: shift.end.time.to_string(),
                                location: Location { lat: 0.0, lng: 0.0 },
                            }), //optional
                            breaks: None, // TODO: expose breaks
                            reloads: None,
                        })
                        .collect(),
                    capacity: vec![vehicle.capacity],
                    skills: None, // TODO: expose some skills
                    limits: None, // TODO: more on all of these
                }
            })
            .collect();

        // TODO: explain single profile and provide valid inputs
        let profile = Profile {
            name: "car".to_string(),
            profile_type: "car".to_string(),
            speed: None,
        }; //TODO: enum this

        vrp_pragmatic::format::problem::Problem {
            plan: vrp_pragmatic::format::problem::Plan {
                jobs: vec![],
                relations: None,
            },
            fleet: vrp_pragmatic::format::problem::Fleet {
                vehicles: fleet,
                profiles: vec![profile],
            },
            objectives: None,
            config: None,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimpleTrip {
    pub coordinate_vehicles: Vec<String>,
    pub coordinate_jobs: Vec<String>,
}

impl SimpleTrip {
    pub async fn convert_to_internal_problem(&self) -> problem::Problem {
        problem::Problem {
            plan: ProblemPlan {
                jobs: self.build_jobs(),
                relations: None,
            },
            fleet: ProblemFleet {
                vehicles: self.build_vehicles(),
                profiles: vec![self.get_simple_profile()],
            },
            objectives: None,
            config: None,
        }
    }

    fn get_simple_profile(&self) -> Profile {
        Profile {
            name: "car".to_string(),
            profile_type: "car".to_string(),
            speed: None,
        }
    }

    fn build_jobs(&self) -> Vec<ProblemJob> {
        const JOB_LENGTH: f64 = 120.0;

        self.coordinate_jobs.to_vec()
            .into_par_iter()
            .enumerate()
            .map(|(index, location)| {
                ProblemJob {
                    id: index.to_string(),
                    // TODO [#21]: potentially switch on the type of job to decide whether its a pickup, delivery or service
                    pickups: None,
                    deliveries: None,
                    replacements: None,
                    services: Some(vec![JobTask {
                        places: vec![JobPlace {
                            location: geocoding::lookup_coordinates(location),
                            // TODO [#23]: add constants to this duration
                            // TODO [#24]: parameterise duration for the simple type as an optional query parameter
                            duration: Duration::minutes(JOB_LENGTH as i64).num_seconds() as f64,
                            times: None,
                        }],
                        demand: None,
                        tag: Some(String::from("Simple 120 minute task")),
                    }]),
                    priority: None,
                    skills: None,
                }
            })
            .collect()
    }

    fn build_vehicles(&self) -> Vec<VehicleType> {
        self.coordinate_vehicles.to_vec()
            .into_par_iter()
            .enumerate()
            .map(|(i, vehicle)| {
                VehicleType {
                    type_id: i.to_string(),
                    // TODO: type_id: "car".to_string(),
                    vehicle_ids: vec![i.to_string()],
                    profile: "car".to_string(),
                    costs: VehicleCosts {
                        fixed: None,
                        distance: 0.0,
                        time: 0.0,
                    },
                    shifts: vec![VehicleShift {
                        start: VehiclePlace {
                            time: chrono::Utc::now().to_rfc3339(),
                            location: geocoding::lookup_coordinates(vehicle.to_string()),
                        },
                        end: None,
                        breaks: None,
                        reloads: None,
                    }],
                    capacity: vec![5],
                    skills: None,
                    limits: None,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::request::DetailedRequest;

    #[test]
    fn test() {
        let request = r#"
    {
  "plan": {
    "jobs": [
      {
        "id": "multi_job1",
        "deliveries": [
          {
            "places": [
              {
                "postcode": "",
                "duration": 240.0
              }
            ],
            "priority": 2,
            "properties": ["d1"]
          }
        ]
      }
    ]
  },
  "fleet": {
    "vehicles": [
      {
        "vehicleIds": [
          "vehicle_1"
        ],
        "costs": {
          "fixed": 22.0,
          "distance": 0.0002,
          "time": 0.004806
        },
        "shifts": [
          {
            "start": {
              "time": "2019-07-04T09:00:00Z",
              "postcode": ""
            },
            "end": {
              "time": "2019-07-04T18:00:00Z",
              "postcode": ""
            }
          }
        ],
        "capacity": 10
      }
    ]
  }
}"#;
        let obj: DetailedRequest =
            serde_json::from_str(request).expect("Unable to serialise request");
        assert_eq!(obj.fleet.vehicles[0].costs.fixed, 22.0 as f64);

        let problem = obj.convert_to_internal_problem();
        assert_eq!(problem.fleet.profiles.first().unwrap().name, "car")
    }
}
