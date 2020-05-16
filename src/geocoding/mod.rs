use std::fs::File;

use csv::{Reader, StringRecord};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Geocoding {
    pub query: GeocodingKind
}

#[derive(Deserialize)]
pub enum GeocodingKind {
    POSTCODE(String),
    COORDINATES(Vec<f64>),
}

// let mut cache = LruCache::new(1);
// cache.put("postcodes", build_geocoding_csv());
//
// // let forward_geocoding_postcodes = postcodes.clone();
// // let reverse_geocoding_postcodes = postcodes.clone();
//
// let (mut sender, mut receiver) = channel(100);
// sender.send(cache.get(&"postcodes").unwrap());

// cached!{
//     POSTCODES;
//     fn build_geocoding_csv() -> Reader<File> = {
//         let records = csv::Reader::from_path("postcodes.csv").expect("Issue reading postcodes.csv").;
//         records
//     }
// }

pub fn search_coordinates(query: &str) -> String {
    let lat_index = 1;
    let lon_index = 2;
    let res: StringRecord = build_geocoding_csv()
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

pub fn search_postcode(lat_lon: Vec<f64>) -> String {
    let postcode_index = 1;
    let res: StringRecord = build_geocoding_csv()
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

fn build_geocoding_csv() -> Reader<File> {
    csv::Reader::from_path("postcodes.csv").expect("Issue reading postcodes.csv")
}
