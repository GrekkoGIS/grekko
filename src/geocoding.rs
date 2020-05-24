use std::fs::File;

use csv::{ByteRecord, Reader};
use serde::Deserialize;
use vrp_pragmatic::format::Location;
use cached::macros::*;
use crate::{redis_manager};

#[derive(Deserialize)]
pub struct Geocoding {
    pub query: GeocodingKind,
}

#[derive(Deserialize)]
pub enum GeocodingKind {
    POSTCODE(String),
    COORDINATES(Vec<f64>),
}

cached!{
    POSTCODES;
    fn bootstrap_cache(table: String) -> bool = {
        let table = table.to_string();
        let string_postcode_table = POSTCODE_TABLE_NAME.to_string();
        match table {
            string_postcode_table => {
                // I don't want to read these again to UTF so using a known const
                let postcode_csv_size = 2628568;
                let mut reader = crate::geocoding::read_geocoding_csv();
                let count = redis_manager::count(POSTCODE_TABLE_NAME);

                if count != postcode_csv_size {
                    println!("Bootstrapping postcode cache");
                    redis_manager::bulk_set(&mut reader);
                    true
                } else {
                    true
                }
            }
            _ => {
                println!("No available table named {}", table);
                false
            }
        }
    }
}

pub const POSTCODE_TABLE_NAME: &str = "POSTCODE";
pub const COORDINATES_SEPARATOR: &str = ";";

pub fn lookup_coordinates(query: String) -> Location {
    let coordinates: String = reverse_search(query.clone());
    let coordinates: Vec<&str> = coordinates.split(';').collect();
    Location {
        lat: coordinates[0].parse().expect(format!("There weren't enough coordinates to extract latitude for postcode {}", query).as_str()),
        lng: coordinates[1].parse().expect(format!("There weren't enough coordinates to extract longitude for postcode {}", query).as_str()),
    }
}

pub fn get_postcodes() -> bool {
    bootstrap_cache(POSTCODE_TABLE_NAME.to_string())
}

pub fn reverse_search(query: String) -> String {
    if get_postcodes() {
        match reverse_search_cache(query) {
            Some(value) => value,
            None => String::from("EMPTY") //TODO this is a poop error message
        }
    } else {
        reverse_search_file(query)
    }
}

pub fn reverse_search_cache(query: String) -> Option<String> {
    // TODO sort this out, rust doesnt like fluent
    let postcode = query;
    let postcode = postcode.replace(" ", "");
    let postcode = postcode.replace("-", "");
    let postcode = postcode.replace(",", "");
    let postcode = postcode.replace(COORDINATES_SEPARATOR, "");
    let postcode = postcode.as_str();

    redis_manager::get_coordinates(postcode)
}

pub fn reverse_search_file(query: String) -> String {
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
    if get_postcodes() {
        forward_search_cache(lat_long)
    } else {
        forward_search_file(lat_long)
    }
}

pub fn forward_search_cache(lat_long: Vec<f64>) -> String {
    match redis_manager::get_postcode(lat_long) {
        None => String::from("Postcode couldn't be found"),
        Some(value) => value,
    }
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

pub fn read_geocoding_csv() -> Reader<File> {
    csv::Reader::from_path("postcodes.csv").expect("Issue reading postcodes.csv")
}

#[cfg(test)]
mod tests {
    use crate::geocoding::{bootstrap_cache, forward_search_file, reverse_search_file, get_postcodes};
    use grekko::get_postcodes;

    #[test]
    fn test_search_postcode() {
        let coordinates = vec![57.099011, -2.252854];
        let postcode = forward_search_file(coordinates);
        assert_eq!(postcode, "AB10AJ")
    }

    #[test]
    fn test_search_coordinates() {
        let coordinates = reverse_search_file(String::from("AB1-0AJ"));
        assert_eq!(coordinates, "57.099011;-2.252854")
    }

    #[test]
    fn test_bootstrap_postcode_cache() {
        get_postcodes();
    }
}
