use std::error::Error;
use std::fs::File;
use std::sync::{Arc, Mutex};

use csv::{Reader, StringRecord};

pub fn search(postcodes: &mut Reader<File>, query: &str) -> (String, String) {
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
    (res.get(lat_index).unwrap().to_owned(), res.get(lon_index).unwrap().to_owned())
}
