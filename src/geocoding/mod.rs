use std::fs::File;

use csv::{ByteRecord, Reader, StringRecord};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Geocoding {
    pub query: GeocodingKind,
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

// TODO this memoisation
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
    let res: ByteRecord = build_geocoding_csv()
        .byte_records()
        .find(|record| {
            record
                .as_ref()
                .expect("Couldn't serialise record to a byte record")
                .iter()
                .any(|field| field == query.as_bytes())
        })
        .unwrap_or_else(|| panic!("Unable to find coordinates for {}", query))
        .expect("Find result could not be unwrapped!");

    format!(
        "{};{}",
        std::str::from_utf8(res.get(lat_index).unwrap().to_owned().as_ref())
            .expect("Unable to unwrap latitude"),
        std::str::from_utf8(res.get(lon_index).unwrap().to_owned().as_ref())
            .expect("Unable to unwrap longitude")
    )
}

pub fn search_postcode(lat_lon: Vec<f64>) -> String {
    let postcode_index = 0;
    let res: ByteRecord = build_geocoding_csv()
        .byte_records()
        .find(|record| {
            record
                .as_ref()
                .expect("Couldn't serialise record to a byte record")
                .iter()
                .any(|field| field == lat_lon.first().unwrap().to_string().as_bytes())
        })
        .unwrap_or_else(|| panic!("Unable to find a postcode for {:?}", lat_lon))
        .expect("Find result could not be unwrapped!");
    String::from_utf8(res.get(postcode_index).unwrap().to_owned())
        .expect("Unable to unwrap postcode")
}

fn build_geocoding_csv() -> Reader<File> {
    csv::Reader::from_path("postcodes.csv").expect("Issue reading postcodes.csv")
}

#[cfg(test)]
mod tests {
    use crate::geocoding::{search_coordinates, search_postcode};

    #[test]
    fn test_search_postcode() {
        let coordinates = vec![57.099011, -2.252854];
        let postcode = search_postcode(coordinates);
        assert_eq!(postcode, "AB1 0AJ")
    }

    #[test]
    fn test_search_coordinates() {
        let coordinates = search_coordinates("AB1 0AJ");
        assert_eq!(coordinates, "57.099011;-2.252854")
    }
}
