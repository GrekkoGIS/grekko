use serde::{Deserialize, Serialize};
use itertools::Itertools;

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
    // let coords = coordinates.join(";");
    // let coords = coords.join(",");

    let crds: String = coordinates.iter().map(|c| {
        let coords_cols_str: Vec<String> = c.iter().map(ToString::to_string).collect();
        coords_cols_str.join(",")
    })
        .collect::<Vec<String>>()
        .join(";");

    let client = reqwest::Client::new();
    let response_body = client.get("https://api.mapbox.com/directions-matrix/v1/mapbox/driving/-122.42,37.78;-122.45,37.91;-122.48,37.73")
        .query(&[
            ("access_token", ""),
            ("annotations", "distance")
        ])
        .send()
        .await.ok()?
        .text()
        .await.ok()?;
    let matrix: Matrix = serde_json::from_str(response_body.as_str()).ok()?;
    Some(matrix)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_matrix() {
        let code = get_matrix(vec![vec![0.0, 0.0]]).await.unwrap().code;
        assert_eq!(code, "Ok")
    }
}