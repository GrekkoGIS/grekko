use std::fs::File;

use csv::{ByteRecord, Reader};
use serde::Deserialize;
use vrp_pragmatic::format::Location;

use crate::redis_manager;
use failure::Error;
use failure::_core::convert::Infallible;

#[derive(Deserialize)]
pub struct Geocoding {
    pub query: GeocodingKind,
}

#[derive(Deserialize)]
pub enum GeocodingKind {
    POSTCODE(String),
    COORDINATES(Vec<f64>),
}

cached! {
    POSTCODES;
    fn bootstrap_cache(table: String) -> bool = {
        match table.as_str() {
            POSTCODE_TABLE_NAME => {
                // I don't want to read these again to UTF so using a known const
                let postcode_csv_size = 2628568;
                let mut reader = crate::geocoding::read_geocoding_csv();
                let count = redis_manager::count(POSTCODE_TABLE_NAME);

                if count < postcode_csv_size {
                    log::info!("Bootstrapping postcode cache");
                    redis_manager::bulk_set(&mut reader, POSTCODE_TABLE_NAME);
                    true
                } else {
                    log::info!("Postcode cache was already bootstrapped");
                    true
                }
            }
            _ => {
                log::error!("No available table named {}", table);
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
        lat: coordinates[0].parse().unwrap_or_else(|_| {
            //TODO remove these panic and add to an unassigned list
            log::error!(
                "There weren't enough coordinates to extract latitude for postcode {}",
                query
            );
            0.0
        }),
        lng: coordinates[1].parse().unwrap_or_else(|_| {
            log::error!(
                "There weren't enough coordinates to extract longitude for postcode {}",
                query
            );
            0.0
        }),
    }
}

pub fn get_postcodes() -> bool {
    bootstrap_cache(POSTCODE_TABLE_NAME.to_string())
}

pub fn reverse_search(query: String) -> String {
    if get_postcodes() {
        match reverse_search_cache(query) {
            Ok(value) => value,
            Err(_) => String::from("EMPTY"), //TODO this is a poop error message
        }
    } else {
        reverse_search_file(query)
    }
}

pub fn reverse_search_cache(query: String) -> Result<String, Error> {
    let postcode = build_cache_key(query);
    let postcode = postcode.as_str();
    redis_manager::get_coordinates(postcode)
}

fn build_cache_key(query: String) -> String {
    // TODO [#39]: sort this out, rust doesn't like fluent that much
    let postcode = query;
    let postcode = postcode.replace(" ", "");
    let postcode = postcode.replace("-", "");
    let postcode = postcode.replace(",", "");
    postcode.replace(COORDINATES_SEPARATOR, "")
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
        Err(err) => String::from("Postcode couldn't be found"),
        Ok(value) => value,
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

pub async fn receive_and_search_coordinates(
    _token: String,
    postcode: String,
) -> Result<impl warp::Reply, Infallible> {
    // get_user_from_token(token).await.unwrap();
    let result = reverse_search(postcode);
    Ok(result)
}

pub async fn receive_and_search_postcode(
    lat: f64,
    lon: f64,
    _token: String,
) -> Result<impl warp::Reply, Infallible> {
    // get_user_from_token(token).await.unwrap();
    let result = forward_search(vec![lat, lon]);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::geocoding::{
        build_cache_key, forward_search_file, get_postcodes, reverse_search_file,
        COORDINATES_SEPARATOR,
    };

    #[test]
    fn test_search_postcode() {
        let coordinates = vec![57.099011, -2.252854];
        let postcode = forward_search_file(coordinates);
        assert_eq!(postcode, "AB1 0AJ")
    }

    #[test]
    fn test_search_coordinates() {
        let coordinates = reverse_search_file(String::from("AB1-0AJ"));
        assert_eq!(coordinates, "57.099011;-2.252854")
    }

    #[test]
    fn test_bootstrap_postcode_cache() {
        assert_eq!(get_postcodes(), true);
    }

    #[test]
    fn test_build_cache_key() {
        let key = "IMAGINARY; -,POSTCODE";
        let key = build_cache_key(String::from(key));
        assert!(!key.contains(' '));
        assert!(!key.contains('-'));
        assert!(!key.contains(','));
        assert!(!key.contains(COORDINATES_SEPARATOR));
        assert_eq!(key, "IMAGINARYPOSTCODE")
    }
}
