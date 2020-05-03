use std::sync::Arc;

use serde::{Deserialize, Serialize};
use vrp_core::models::Problem;
use vrp_pragmatic::format::Location;
use vrp_pragmatic::format::problem::{VehiclePlace, VehicleShift, Profile};

// imports both the trait and the derive macro

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
        let fleet = self.fleet.vehicles
            .iter()
            .map(|vehicle| {
                vrp_pragmatic::format::problem::VehicleType {
                    type_id: "vehicle".to_owned(), //TODO understand this
                    vehicle_ids: (*vehicle.vehicle_ids).to_owned(),
                    profile: "car".to_string(), //TODO enum this
                    costs: vrp_pragmatic::format::problem::VehicleCosts {
                        fixed: Option::from(vehicle.costs.fixed),
                        distance: vehicle.costs.distance,
                        time: vehicle.costs.time,
                    },
                    shifts: vehicle.shifts
                        .iter()
                        .map(|shift| VehicleShift {
                            start: VehiclePlace { time: shift.start.time.to_string(), location: Location { lat: 0.0, lng: 0.0 } }, //TODO convert these
                            end: None, //optional
                            breaks: None,
                            reloads: None,
                        }).collect(),
                    capacity: vec![vehicle.capacity],
                    skills: None,
                    limits: None, //TODO more on all of these
                }
            }).collect();

        //TODO explain single profile
        let profile = Profile {
            name: "car".to_string(),
            profile_type: "car".to_string()
        }; //TODO enum this

        vrp_pragmatic::format::problem::Problem {
            plan: vrp_pragmatic::format::problem::Plan { jobs: vec![], relations: None },
            fleet: vrp_pragmatic::format::problem::Fleet { vehicles: fleet, profiles: vec![profile] },
            objectives: None,
            config: None,
        }
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
        let obj: DetailedRequest = serde_json::from_str(request).expect("Unable to serialise request");
        assert_eq!(obj.fleet.vehicles[0].costs.fixed, 22.0);

        let problem = obj.convert_to_internal_problem();
        assert_eq!(problem.fleet.profiles.first().unwrap().name, "car")
    }
}