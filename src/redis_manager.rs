use std::fs::File;

use crate::geocoding::{COORDINATES_SEPARATOR, POSTCODE_TABLE_NAME};
use csv::Reader;
use redis::{Client, Commands, RedisResult, Connection};
use serde::de::DeserializeOwned;
use serde::export::fmt::Display;
use serde::Serialize;


fn connect_and_query<F, T>(mut action: F) -> Option<T>
    where F: FnMut(Connection) -> Option<T>
{
    let client: Client = get_redis_client().ok()?;
    let con = client.get_connection().ok()?;
    action(con)
}

// TODO [#30]: add concurrency to all of this once benchmarked
fn get_redis_client() -> RedisResult<Client> {
    redis::Client::open("redis://127.0.0.1/")
}

pub fn get_coordinates(postcode: &str) -> Option<String> {
    connect_and_query(|mut connection| {
        connection.hget(POSTCODE_TABLE_NAME, postcode).ok()?
    })
}

pub fn get_postcode(coordinates: Vec<f64>) -> Option<String> {
    let coord_string = coordinates
        .iter()
        .map(|coord| coord.to_string())
        .collect::<Vec<String>>()
        .join(COORDINATES_SEPARATOR);

    // TODO [#31]: fix this
    connect_and_query(|mut connection| {
        redis::cmd("HSCAN")
            .arg(&["0", "MATCH", &coord_string])
            .query(&mut connection)
            .ok()?
    })
}

pub fn get<T: DeserializeOwned>(table: &str, key: &str) -> Option<T> {
    let result: Option<String> = connect_and_query(|mut connection| {
        connection.hget(table, key).ok()?
    });

    match result {
        None => None,
        Some(res) => serde_json::from_str(res.as_str()).ok()?,
    }
}

pub fn set<T: Serialize + Display>(table: &str, key: &str, value: T) -> Option<String> {
    let client: Client = get_redis_client().expect("Unable to get a redis client");
    let mut con = client.get_connection().expect("Unable to get a connection");

    let result: RedisResult<i32> = con.hset(
        table,
        key,
        serde_json::to_string(&value).expect("Unable to serialize value")
    );

    match result {
        Err(err) => {
            eprintln!("Couldn't write to redis, reason: {:?}", err.detail());
            None
        },
        Ok(res) => {
            let msg = format!("Wrote {} to table: {} with key {} and result {}", value, table, key, res);
            println!("{}", msg);
            Some(msg)
        },
    }
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

    let mut count = 0;
    let mut pipeline = redis::pipe();

    // TODO [#32]: use rayon to parallelise this
    records.for_each(|row| {
        let row = &row.unwrap();
        count += 1;
        pipeline
            .hset(
                POSTCODE_TABLE_NAME,
                row.get(postcode_index)
                    .unwrap()
                    .to_string()
                    .replace(" ", ""),
                format!(
                    "{};{}",
                    row.get(lat_index).unwrap(),
                    row.get(lon_index).unwrap()
                ),
            )
            .ignore();
    });

    let result: RedisResult<i32> = pipeline.query(&mut con);

    println!(
        "Finished bootstrapping {} postcodes, result: {:?}",
        count, result
    );
}
