use std::error::Error;
use std::fs::File;
use std::sync::{Arc, Mutex};

use csv::{Reader, StringRecord};

struct Postcodes {
    reader: Reader<File>
}

pub fn main() {
    // Build CSV readers and writers to stdin and stdout, respectively.

    // Iterate over all the records in `RDR`, and write only records containing
    // `query` to `wtr`.
    // for result in POSTCODES.reader.records() {
    //     let record = result.expect("Result cant be unmarshalled to a record");
    //     if record.iter().any(|field| field == "BS1 1AA") {
    //         println!("{:?} {:?}", record.get(1), record.get(2))
    //     }
    // }
}



pub fn search(postcodes: &mut Reader<File>, query: &str) -> (String, String) {
    let res: &StringRecord = &postcodes
        .records()
        .find(
            |record| record.as_ref()
                .expect("Stringrecord couldnt be found")
                .iter()
                .any(|field| field == query),
        )
        .expect("Unable to find query").expect("");
    (res.get(1).unwrap().to_owned(), res.get(2).unwrap().to_owned())
}
