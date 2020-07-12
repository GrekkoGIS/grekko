use chrono::Duration;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use vrp_pragmatic::format::problem::Fleet as ProblemFleet;
use vrp_pragmatic::format::problem::Job as ProblemJob;
use vrp_pragmatic::format::problem::Plan as ProblemPlan;
use vrp_pragmatic::format::problem::{
    JobPlace, JobTask, Profile, VehicleCosts, VehiclePlace, VehicleShift, VehicleType,
};
use vrp_pragmatic::format::{problem, Location};

use crate::geocoding;
use failure::Error;

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
                                // TODO [#33]: utilise geocoding to get coordinates
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

        // TODO [#34]: explain single profile and provide valid inputs
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
    pub async fn convert_to_internal_problem(&self) -> Result<problem::Problem, Error> {
        Ok(problem::Problem {
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
        })
    }

    fn get_simple_profile(&self) -> Profile {
        const FOURTY_MPH_IN_METRES_PER_SECOND: f64 = 17.0;
        let normal_car = "normal_car".to_string();
        let car_type = "car".to_string();
        Profile {
            name: normal_car,
            profile_type: car_type,
            speed: Some(FOURTY_MPH_IN_METRES_PER_SECOND), // TODO: average 40mph
        }
    }

    fn build_jobs(&self) -> Vec<ProblemJob> {
        const JOB_LENGTH: f64 = 120.0;

        let (locations, errors): (Vec<_>, Vec<_>) = self
            .coordinate_jobs
            .to_vec()
            .into_iter()
            .map(|location| geocoding::lookup_coordinates(location))
            .partition(Result::is_ok);
        let locations: Vec<Location> = locations.into_iter().map(Result::unwrap).collect();
        let errors: Vec<failure::Error> = errors.into_iter().map(Result::unwrap_err).collect();

        locations
            .to_vec()
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
                            location, // TODO: fix this unwrap
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
        let (locations, errors): (Vec<_>, Vec<_>) = self
            .coordinate_vehicles
            .to_vec()
            .into_iter()
            .map(|location| geocoding::lookup_coordinates(location))
            .partition(Result::is_ok);
        let locations: Vec<Location> = locations.into_iter().map(Result::unwrap).collect();
        let errors: Vec<failure::Error> = errors.into_iter().map(Result::unwrap_err).collect();

        locations
            .to_vec()
            .into_par_iter()
            .enumerate()
            .map(|(i, vehicle)| {
                VehicleType {
                    type_id: i.to_string(),
                    // TODO [#35]: type_id: "car".to_string(), for some reason this needs to be unique?
                    vehicle_ids: vec![i.to_string()],
                    profile: "normal_car".to_string(),
                    costs: VehicleCosts {
                        fixed: Some(22.0),
                        distance: 0.0002,
                        time: 0.004806,
                    },
                    shifts: vec![VehicleShift {
                        start: VehiclePlace {
                            time: chrono::Utc::now().to_rfc3339(),
                            location: vehicle,
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
    use crate::request::{DetailedRequest, SimpleTrip};

    #[test]
    fn test_deserialise_and_convert() {
        let request = r#"{"coordinate_vehicles": ["BS1 3AA", "BA2 1AA"],"coordinate_jobs": ["BS6 666", "BS7 777"]}"#;

        let obj: SimpleTrip = serde_json::from_str(request).unwrap();
        assert_eq!(obj.coordinate_vehicles.first().unwrap(), "BS1 3AA");
        assert_eq!(obj.coordinate_vehicles[1], "BA2 1AA");
        assert_eq!(obj.coordinate_jobs.first().unwrap(), "BS6 666");
        assert_eq!(obj.coordinate_jobs[1], "BS7 777");
    }

    #[test]
    fn test_deserialise_and_build_vehicles() {
        let request = r#"{"coordinate_vehicles": ["BS1 3AA", "BA2 1AA"],"coordinate_jobs": ["BS6 666", "BS7 777"]}"#;

        let obj: SimpleTrip = serde_json::from_str(request).unwrap();

        let vehicles = obj.build_vehicles();

        assert_eq!(
            vehicles.first().unwrap().vehicle_ids.first().unwrap(),
            &0.to_string()
        );
        assert_eq!(vehicles.first().unwrap().type_id, 0.to_string());
        assert_eq!(vehicles.first().unwrap().profile, "normal_car".to_string());
        assert_eq!(vehicles.first().unwrap().costs.fixed, Some(22.0));
        assert_eq!(vehicles.first().unwrap().costs.distance, 0.0002);
        assert_eq!(vehicles.first().unwrap().costs.time, 0.004806);
        assert_eq!(vehicles.first().unwrap().capacity.first().unwrap(), &5);
        assert_eq!(
            vehicles
                .first()
                .unwrap()
                .shifts
                .first()
                .unwrap()
                .start
                .location
                .lat,
            51.455691
        );
        assert_eq!(
            vehicles
                .first()
                .unwrap()
                .shifts
                .first()
                .unwrap()
                .start
                .location
                .lng,
            -2.586119
        );

        assert_eq!(vehicles[1].vehicle_ids.first().unwrap(), &1.to_string());
        assert_eq!(vehicles[1].type_id, 1.to_string());
        assert_eq!(vehicles[1].profile, "normal_car".to_string());
        assert_eq!(vehicles[1].costs.fixed, Some(22.0));
        assert_eq!(vehicles[1].costs.distance, 0.0002);
        assert_eq!(vehicles[1].costs.time, 0.004806);
        assert_eq!(vehicles[1].capacity.first().unwrap(), &5);
        assert_eq!(
            vehicles[1].shifts.first().unwrap().start.location.lat,
            51.375932
        );
        assert_eq!(
            vehicles[1].shifts.first().unwrap().start.location.lng,
            -2.382291
        );
    }

    #[test]
    fn test_deserialise_and_build_jobs() {
        let request = r#"{"coordinate_vehicles": ["BS13AA", "BA21AA"],"coordinate_jobs": ["BS11AA", "BA21AA"]}"#;

        let obj: SimpleTrip = serde_json::from_str(request).unwrap();

        let jobs = obj.build_jobs();

        let service = jobs[0].services.clone().unwrap();
        assert_eq!(jobs[0].id, 0.to_string());
        assert_eq!(service[0].places[0].location.lat, 51.449516);
        assert_eq!(service[0].places[0].location.lng, -2.57837);
        assert_eq!(service[0].places[0].duration, 7200.0);

        assert_eq!(jobs[1].id, 1.to_string());
        let service = jobs[1].services.clone().unwrap();
        assert_eq!(service[0].places[0].location.lat, 51.375932);
        assert_eq!(service[0].places[0].location.lng, -2.382291);
        assert_eq!(service[0].places[0].duration, 7200.0);
    }

    #[test]
    fn test_convert_to_internal_problem() {
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
