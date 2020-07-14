use std::fs::File;

use csv::{ByteRecord, Reader};
use failure::_core::convert::Infallible;
use failure::{Error, ResultExt};
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

pub const POSTCODE_TABLE_NAME: &str = "POSTCODE";
pub const COORDINATES_SEPARATOR: &str = ";";

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

pub fn get_location_from_postcode(query: &String) -> Result<Location, Error> {
    let get_from_geo_command = true;
    let key_sanitized = build_cache_key(&query);
    let (lng, lat) = if get_from_geo_command {
        get_location_from_geo_ops(&key_sanitized)?
    } else {
        get_location_from_table(&key_sanitized)?
    };
    Ok(map_to_location(lng, lat))
}
fn get_location_from_table(query: &String) -> Result<(f64, f64), Error> {
    let coordinates: String = reverse_search(&query)?;
    let coordinates: Vec<&str> = coordinates.split(';').collect();
    Ok((coordinates[0].parse()?, coordinates[1].parse()?))
}
fn get_location_from_geo_ops(query: &str) -> Result<(f64, f64), Error> {
    Ok(redis_manager::get_geo_pos(query)?)
}
fn map_to_location(lng: f64, lat: f64) -> Location {
    Location { lng, lat }
}

pub fn reverse_search(query: &str) -> Result<String, Error> {
    let coordinate_string = if is_bootstrapped() {
        reverse_search_cache_table(&query)
    } else {
        reverse_search_file(&query)
    }?;
    Ok(check_coordinate_string(&query, coordinate_string)?)
}

fn check_coordinate_string(query: &str, coordinates: String) -> Result<String, Error> {
    if coordinates == String::from("99.999999;0.000000") {
        let msg = format!("Location is invalid for: {:?}", query);
        log::error!("{}", msg);
        return Err(failure::err_msg(msg));
    } else {
        Ok(coordinates)
    }
}

pub fn reverse_search_cache_table(postcode: &str) -> Result<String, Error> {
    redis_manager::get_coordinates(postcode).map_err(|err| {
        log::error!("Failed to get coordinates for {}", err);
        err
    })
}
pub fn reverse_search_file(query: &str) -> Result<String, Error> {
    let lat_index = 1;
    let lon_index = 2;
    let res = read_geocoding_csv()
        .byte_records()
        .find(|record| {
            record
                .as_ref()
                .expect("Couldn't serialise record to a byte record") // TODO: remove
                .iter()
                .any(|field| field == query.replace(" ", "").replace("-", " ").as_bytes())
        })
        .ok_or(failure::err_msg(format!(
            "Unable to find coordinates for {}",
            query
        )))?;

    let record = res?;
    Ok(format!(
        "{};{}",
        std::str::from_utf8(record.get(lat_index).unwrap().to_owned().as_ref())?,
        // .expect("Unable to unwrap latitude"),
        std::str::from_utf8(record.get(lon_index).unwrap().to_owned().as_ref())? // .expect("Unable to unwrap longitude")
    ))
}

pub fn forward_search(lat_long: Vec<f64>) -> String {
    if is_bootstrapped() {
        forward_search_cache_table(lat_long)
    } else {
        forward_search_file(lat_long)
    }
}
pub fn forward_search_cache_table(lat_long: Vec<f64>) -> String {
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
        .unwrap_or_else(|| panic!("Unable to find a postcode for {:?}", lat_lon)) // TODO: dont ever panic
        .expect("Find result could not be unwrapped!");
    String::from_utf8(res.get(postcode_index).unwrap().to_owned())
        .expect("Unable to unwrap postcode")
}

pub fn is_bootstrapped() -> bool {
    bootstrap_cache(POSTCODE_TABLE_NAME.to_string())
}
fn build_cache_key(query: &String) -> String {
    // TODO [#39]: sort this out, rust doesn't like fluent that much
    let postcode = query;
    let postcode = postcode.replace(" ", "");
    let postcode = postcode.replace("-", "");
    let postcode = postcode.replace(",", "");
    postcode.replace(COORDINATES_SEPARATOR, "")
}

pub fn read_geocoding_csv() -> Reader<File> {
    csv::Reader::from_path("postcodes.csv").expect("Issue reading postcodes.csv")
}

#[cfg(test)]
mod tests {
    use crate::geocoding::{
        build_cache_key, forward_search_file, is_bootstrapped, reverse_search_file,
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
        let coordinates = reverse_search_file(&String::from("AB1-0AJ"));
        assert_eq!(coordinates, "57.099011;-2.252854")
    }

    #[test]
    fn test_bootstrap_postcode_cache() {
        assert_eq!(is_bootstrapped(), true);
    }

    #[test]
    fn test_build_cache_key() {
        let key = "IMAGINARY; -,POSTCODE";
        let key = build_cache_key(&String::from(key));
        assert!(!key.contains(' '));
        assert!(!key.contains('-'));
        assert!(!key.contains(','));
        assert!(!key.contains(COORDINATES_SEPARATOR));
        assert_eq!(key, "IMAGINARYPOSTCODE")
    }
}
