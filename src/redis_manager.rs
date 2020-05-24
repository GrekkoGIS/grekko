use std::fs::File;

use csv::Reader;
use redis::{Client, Commands, RedisResult};
// TODO: add concurrency to all of this once benchmarked
fn get_redis_client() -> RedisResult<Client> {
    redis::Client::open("redis://127.0.0.1/")
}

pub fn get_coordinates(postcode: &str) -> Option<String> {
    let client: Client = get_redis_client().ok()?;
    let mut con = client.get_connection().ok()?;

    con.get(postcode).ok()?
}

pub fn get_postcode(coordinates: Vec<f64>) -> Option<String> {
    let client: Client = get_redis_client().ok()?;
    let mut con = client.get_connection().ok()?;

    let coord_string = coordinates
        .iter()
        .map(|coord| coord.to_string())
        .collect::<Vec<String>>()
        .join(";");

    redis::cmd("HSCAN").arg(&["0", "MATCH", &coord_string]).query(&mut con).ok()?
    // con.get(postcode).ok()?
}

pub fn count(table: &str) -> i32 {
    let client: Client = get_redis_client().unwrap();
    let mut con = client.get_connection().unwrap();

    con.hlen(table).unwrap()
}

pub fn bulk_set(reader: &mut Reader<File>) {
    let records = reader.records();
    let client: Client = get_redis_client().unwrap();
    let mut con = client.get_connection().unwrap();

    let postcode_index = 0;
    let lat_index = 1;
    let lon_index = 2;
    let table_name = "POSTCODE";

    let mut count = 0;
    let mut pipeline = redis::pipe();

    // TODO use rayon to parallelise this
    records.for_each(|row| {
        let row = &row.unwrap();
        count += 1;
        &pipeline
            .hset(
                table_name,
                row.get(postcode_index).unwrap().to_string().replace(" ", ""),
                format!("{};{}", row.get(lat_index).unwrap(), row.get(lon_index).unwrap()),
            )
            .ignore();
    }
    );

    let result: RedisResult<i32> = pipeline.query(&mut con);

    println!("Finished bootstrapping {} postcodes, result: {:?}", count, result);
}