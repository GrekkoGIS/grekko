use serde::{Deserialize, Serialize};

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
    pub capacity: i64,
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


mod tests {
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
    }
}