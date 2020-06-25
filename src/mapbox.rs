use std::env;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Matrix {
    pub code: String,
    pub distances: Vec<Vec<f64>>,
    pub destinations: Vec<Destination>,
    pub sources: Vec<Source>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Destination {
    pub distance: f64,
    pub name: String,
    pub location: Vec<f64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub distance: f64,
    pub name: String,
    pub location: Vec<f64>,
}

pub async fn get_matrix(coordinates: Vec<Vec<f64>>) -> Option<Matrix> {
    let coords: String = coordinates.iter().map(|long_lat| {
        let coords_cols_str: Vec<String> = long_lat.iter().map(ToString::to_string).collect();
        coords_cols_str.join(",")
    })
        .collect::<Vec<String>>()
        .join(";");

    let access_token = env::var("MAPBOX_ACCESS_KEY").expect("MAPBOX_ACCESS_KEY isn't set");

    let client = reqwest::Client::new();

    let url = format!("https://api.mapbox.com/directions-matrix/v1/mapbox/driving/{}", coords.as_str());
    let response_body = client.get(&url)
        .query(&[
            ("access_token", access_token.as_str()),
            ("annotations", "distance")
        ])
        .send()
        .await.ok()?
        .text()
        .await.ok()?;

    let matrix: Matrix = serde_json::from_str(response_body.as_str()).ok()?;
    println!("{:?}", matrix);
    Some(matrix)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_matrix() {
        let code = get_matrix(vec![
            vec![-1.080278, 53.958332],
            vec![-2.220000, 52.192001],
            vec![-1.308000, 51.063202]
        ])
            .await.unwrap().code;
        assert_eq!(code, "Ok")
    }
}