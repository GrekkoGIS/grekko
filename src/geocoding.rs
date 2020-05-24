use std::fs::File;

use csv::{ByteRecord, Reader};
use serde::Deserialize;
use vrp_pragmatic::format::Location;
use crate::redis_manager;

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

// TODO [#17]: memoize the cached keys and utilise channels to get it working
// cached!{
//     POSTCODES;
//     fn build_geocoding_csv() -> Reader<File> = {
//         let records = csv::Reader::from_path("postcodes.csv").expect("Issue reading postcodes.csv").;
//         records
//     }
// }

const POSTCODE_TABLE_NAME: &str = "POSTCODE";

pub async fn bootstrap_cache(table: &str) -> bool {

    match table {
        POSTCODE_TABLE_NAME => {
            println!("Bootstrapping postcode cache");

            // I dont want to read these again to UTF so using a known const
            let postcode_csv_size = 2628568;
            let mut reader = read_geocoding_csv();

            let count = redis_manager::count(table).await;

            if count != postcode_csv_size {
                redis_manager::bulk_set(&mut reader).await;
                true
            } else {
                println!("Postcodes have already been bootstrapped");
                true
            }
        },
        _ => {
            println!("No available table named {}", table);
            false
        }
    }
}

pub fn search_location(query: &str) -> Location {
    let coordinates: String = reverse_search_file(query);
    let coordinates: Vec<&str> = coordinates.split(',').collect();
    Location {
        lat: coordinates[0].parse().unwrap(),
        lng: coordinates[1].parse().unwrap(),
    }
}

pub fn reverse_search(query: &str) -> String {
    if bootstrap_cache(POSTCODE_TABLE_NAME) {
        reverse_search_cache(query)
    } else {
        reverse_search_file(query)
    }
}

pub fn reverse_search_cache(query: &str) -> String {
    redis_manager::get(query.replace(" ", "").as_str())
}

pub fn reverse_search_file(query: &str) -> String {
    let lat_index = 1;
    let lon_index = 2;
    let res: ByteRecord = read_geocoding_csv()
        .byte_records()
        .find(|record| {
            record
                .as_ref()
                .expect("Couldn't serialise record to a byte record")
                .iter()
                .any(|field| field == query.replace(" ", "").replace("-", " ").as_bytes())
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


pub fn forward_search(lat_long: Vec<f64>) -> String {
    if bootstrap_cache(POSTCODE_TABLE_NAME) {
        // TODO: forward search the cache
        forward_search_file(lat_long)
    } else {
        forward_search_file(lat_long)
    }
}

pub fn forward_search_cache(lat_long: Vec<f64>) -> String {
    if bootstrap_cache(POSTCODE_TABLE_NAME) {
        // redis_manager::get(lat_long)
    }
    String::new()
}

pub fn forward_search_file(lat_lon: Vec<f64>) -> String {
    let postcode_index = 0;
    let res: ByteRecord = read_geocoding_csv()
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

fn read_geocoding_csv() -> Reader<File> {
    csv::Reader::from_path("postcodes.csv").expect("Issue reading postcodes.csv")
}

#[cfg(test)]
mod tests {
    use crate::geocoding::{reverse_search_file, forward_search_file, bootstrap_cache};

    #[test]
    fn test_search_postcode() {
        let coordinates = vec![57.099011, -2.252854];
        let postcode = forward_search_file(coordinates);
        assert_eq!(postcode, "AB10AJ")
    }

    #[test]
    fn test_search_coordinates() {
        let coordinates = reverse_search_file("AB1-0AJ");
        assert_eq!(coordinates, "57.099011;-2.252854")
    }

    #[test]
    fn test_bootstrap_postcode_cache() {
        bootstrap_cache("POSTCODE");
    }
}
