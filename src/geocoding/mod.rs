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
    let res: StringRecord = build_geocoding_csv()
        .records()
        .find(
            |record| record.as_ref()
                .expect("Couldn't serialise record to a string record")
                .iter()
                .any(|field| field == query),
        )
        .unwrap_or_else(|| panic!("Unable to find coordinates for {}", query))
        .expect("Issue unwrapping find");
    format!("{};{}", res.get(lat_index).unwrap().to_owned(), res.get(lon_index).unwrap().to_owned())
}

pub fn search_postcode(lat_lon: Vec<f64>) -> String {
    let postcode_index = 0;
    let res: StringRecord = build_geocoding_csv()
        .records()
        .find(
            |record| record.as_ref()
                .expect("Couldn't serialise record to a string record")
                .iter()
                .any(|field| field == lat_lon.first().unwrap().to_string()),
        )
        .unwrap_or_else(|| panic!("Unable to find a postcode for {:?}", lat_lon))
        .expect("Issue unwrapping find");
    res.get(postcode_index).unwrap().to_owned()
}

fn build_geocoding_csv() -> Reader<File> {
    csv::Reader::from_path("postcodes.csv").expect("Issue reading postcodes.csv")
}

#[cfg(test)]
mod tests {
    use crate::geocoding::{search_postcode, search_coordinates};

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