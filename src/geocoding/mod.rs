use std::fs::File;

use serde::Deserialize;
use csv::{Reader, StringRecord};

#[derive(Deserialize)]
pub struct Geocoding {
    pub query: GeocodingKind
}

#[derive(Deserialize)]
pub enum GeocodingKind {
    POSTCODE(String),
    COORDINATES(Vec<f64>),
}

pub fn search_coordinates(postcodes: &mut Reader<File>, query: &str) -> String {
    let lat_index = 1;
    let lon_index = 2;
    let res: &StringRecord = &postcodes
        .records()
        .find(
            |record| record.as_ref()
                .expect("Couldn't serialise record to a string record")
                .iter()
                .any(|field| field == query),
        )
        .expect(format!("Unable to find {}", query).as_str())
        .expect("Issue unwrapping find");
    format!("{};{}", res.get(lat_index).unwrap().to_owned(), res.get(lon_index).unwrap().to_owned())
}

pub fn search_postcode(postcodes: &mut Reader<File>, lat_lon: Vec<f64>) -> String {
    let postcode_index = 1;
    let res: &StringRecord = &postcodes
        .records()
        .find(
            |record| record.as_ref()
                .expect("Couldn't serialise record to a string record")
                .iter()
                .any(|field| field == lat_lon.first().unwrap().to_string()),
        )
        .expect(format!("Unable to find {:?}", lat_lon).as_str())
        .expect("Issue unwrapping find");
    format!("{}", res.get(postcode_index).unwrap().to_owned())
}
