use std::fs::File;

use csv::{Reader, StringRecord};
use failure::{Error, ResultExt};
use redis::geo::Coord;
use redis::{Client, Cmd, Commands, Connection, RedisError, RedisResult, Value};
use serde::de::DeserializeOwned;
use serde::export::fmt::Display;
use serde::Serialize;

use crate::geocoding::{COORDINATES_SEPARATOR, POSTCODE_TABLE_NAME};
use rayon::iter::{ParallelBridge, ParallelIterator};
use vrp_pragmatic::format::Location;

fn connect_and_query<F, T>(mut action: F) -> Result<T, Error>
where
    F: FnMut(Connection) -> Result<T, Error>,
{
    let client: Client = get_redis_client().with_context(|err| {
        format!(
            "Failed to  get a redis client, category `{}`",
            err.category()
        )
    })?;
    let con = client
        .get_connection()
        .with_context(|err| format!("Failed to get a connection, category `{}`", err.category()))?;
    action(con)
}

// TODO [#30]: add concurrency to all of this once benchmarked
fn get_redis_client() -> RedisResult<Client> {
    redis::Client::open("redis://127.0.0.1:6375/")
}

pub fn get_geo_pos(key: &str) -> Result<(f64, f64), Error> {
    log::trace!("key {}", key);
    let result: Result<Vec<Vec<f64>>, Error> = connect_and_query(|mut connection| {
        Ok(connection.geo_pos(key, &["UK"]).with_context(|err| {
            format!("Failed to get `{}` from `UK` err `{}`", key, err.category())
        })?)
    });
    log::trace!(
        "Result received from query: `{:?}` from `UK` and key `{}`",
        result,
        key
    );

    match result {
        Err(err) => Err(err.into()),
        Ok(coord_list) => {
            let coords = coord_list.get(0).ok_or(failure::err_msg(format!(
                "Could not get coordinates from {:?}",
                coord_list
            )))?;
            if coords.is_empty() {
                Err(failure::err_msg(format!(
                    "Returned empty coordinates for {}",
                    key
                )))
            } else {
                log::debug!("Coords: {:?}", coords);
                let lng = coords[0];
                let lat = coords[1];
                Ok((lng, lat))
            }
        }
    }
}

pub fn get_coordinates(postcode: &str) -> Result<String, Error> {
    connect_and_query(|mut connection| {
        Ok(connection
            .hget(POSTCODE_TABLE_NAME, postcode)
            .with_context(|err| {
                format!(
                    "Failed to get `{}` from `{}`",
                    postcode, POSTCODE_TABLE_NAME
                )
            })?)
    })
}

pub fn get_postcode(coordinates: Vec<f64>) -> Result<String, Error> {
    let coord_string = coordinates
        .iter()
        .map(|coord| coord.to_string())
        .collect::<Vec<String>>()
        .join(COORDINATES_SEPARATOR);

    // TODO [#31]: fix this
    connect_and_query(|mut connection| {
        Ok(redis::cmd("HSCAN")
            .arg(&["0", "MATCH", &coord_string])
            .query(&mut connection)
            .with_context(|err| {
                format!(
                    "Failed to scan for `{:?}` err `{}`",
                    coordinates,
                    err.category()
                )
            })?)
    })
}

pub fn get<T: DeserializeOwned>(table: &str, key: &str) -> Result<T, Error> {
    let result: Result<String, Error> = connect_and_query(|mut connection| {
        Ok(connection.hget(table, key).with_context(|err| {
            format!(
                "Failed to get `{}` from `{}` err `{}`",
                key,
                table,
                err.category()
            )
        })?)
    });
    log::trace!(
        "Result received from query: `{:?}` for table `{}` and key `{}`",
        result,
        table,
        key
    );

    match result {
        Err(err) => Err(err.into()),
        Ok(res) => Ok(serde_json::from_str(&res)?),
    }
}

pub fn hdel(table: &str, key: &str) -> Result<String, Error> {
    connect_and_query(|mut connection| Ok(connection.hdel(table, key)?))
}

pub fn del(key: &str) -> Result<String, Error> {
    connect_and_query(|mut connection| Ok(connection.del(key)?))
}

pub fn set<T: Serialize + Display>(table: &str, key: &str, value: T) -> Option<String> {
    let client: Client = get_redis_client().expect("Unable to get a redis client");
    let mut con = client.get_connection().expect("Unable to get a connection");

    let result: RedisResult<i32> = con.hset(
        table,
        key,
        serde_json::to_string(&value).expect("Unable to serialize value"),
    );

    match result {
        Err(err) => {
            log::error!("Couldn't write to redis, reason: {:?}", err.detail());
            None
        }
        Ok(res) => {
            let msg = format!(
                "Wrote {} to table: {} with key {} and result {}",
                value, table, key, res
            );
            log::debug!("{}", msg);
            Some(msg)
        }
    }
}

pub fn set_json<T: Serialize + Display>(key: &str, path: Option<&str>, value: T) -> Option<String> {
    let client: Client = get_redis_client().expect("Unable to get a redis client");
    let mut con = client.get_connection().expect("Unable to get a connection");

    let result = if let Some(path) = path {
        let mut cmd = redis::cmd("JSON.SET");
        cmd.arg(key).arg(path).arg(
            serde_json::to_string(&value)
                .with_context(|err| "Unable to serialize value")
                .ok()?,
        );
        cmd
    } else {
        let mut cmd = redis::cmd("JSON.SET");
        cmd.arg(key).arg(".").arg(
            serde_json::to_string(&value)
                .with_context(|err| "Unable to serialize value")
                .ok()?,
        );
        cmd
    };
    let result: RedisResult<String> = result.query(&mut con);

    match result {
        Err(err) => {
            log::error!("Couldn't write to redis, reason: {:?}", err.detail());
            None
        }
        Ok(res) => {
            let msg = format!("Wrote {} with key {} and result {}", value, key, res);
            log::debug!("{}", msg);
            Some(msg)
        }
    }
}
pub fn append_json<T: Serialize>(key: &str, path: &str, value: T) -> Option<String> {
    let client: Client = get_redis_client().expect("Unable to get a redis client");
    let mut con = client.get_connection().expect("Unable to get a connection");

    let value_as_json = serde_json::to_string(&value)
        .with_context(|err| "Unable to serialize value")
        .ok()?;
    let result = {
        let mut cmd = redis::cmd("JSON.ARRAPPEND");
        cmd.arg(key).arg(path).arg(&value_as_json);
        cmd
    };
    let result: RedisResult<i32> = result.query(&mut con);

    match result {
        Err(err) => {
            log::error!("Couldn't write json to redis, reason: {:?}", err.detail());
            None
        }
        Ok(res) => {
            let msg = format!(
                "Wrote {} with key {} and result {:?}",
                value_as_json, key, res
            );
            log::debug!("{}", msg);
            Some(msg)
        }
    }
}

pub fn count(table: &str) -> i32 {
    let client: Client = get_redis_client().unwrap();
    let mut con = client
        .get_connection()
        .with_context(|e| {
            format!(
                "Failed to reach a redis connection. \
                Code: `{:?}`, \
                Detail: `{:?}`, \
                Category: `{:?}`, \
                ClusterError: `{:?}`, \
                ConnectionDropped: `{:?}`, \
                ConnectionRefused: `{:?}`, \
                IOError: `{:?}`, \
                Timeout: `{:?}`",
                e.code(),
                e.detail(),
                e.category(),
                e.is_cluster_error(),
                e.is_connection_dropped(),
                e.is_connection_refusal(),
                e.is_io_error(),
                e.is_timeout(),
            )
        })
        .unwrap();

    con.hlen(table).unwrap()
}

// TODO [#46]: decouple this
pub fn bulk_set(csv: &mut Reader<File>, key: &str) -> Option<()> {
    let records = csv.records();
    let client: Client = get_redis_client().unwrap();
    let mut con = client.get_connection().unwrap();

    let postcode_index = 0;
    let lat_index = 1;
    let lon_index = 2;

    let mut count = 0;
    let mut pipeline = redis::pipe();

    // let s = records
    //     .par_bridge()
    //     .fold(
    //         || redis::pipe(),
    //         |mut pipeline, row| {
    //             let row = &row.unwrap();
    //             count += 1;
    //             let (lon, lat) = build_row_tuple(lat_index, lon_index, row);
    //             if lat != "99.999999" && lon != "0.000000" {
    //                 pipeline
    //                     .geo_add(
    //                         build_row_field(postcode_index, row),
    //                         (Coord::lon_lat(lon, lat), "UK"),
    //                     )
    //                     .ignore();
    //             }
    //             pipeline
    //         },
    //     )
    //     .collect();
    // TODO [#32]: use  rayon to parallelise this
    records.for_each(|row| {
        let row = &row.unwrap();
        count += 1;
        let (lon, lat) = build_row_tuple(lat_index, lon_index, row);
        if lat != "99.999999" && lon != "0.000000" {
            pipeline
                .geo_add(
                    build_row_field(postcode_index, row),
                    (Coord::lon_lat(lon, lat), "UK"),
                )
                .ignore();
        }
    });

    let result: RedisResult<Vec<i32>> = pipeline.query(&mut con);

    match result {
        Ok(res) => {
            log::info!(
                "Finished bootstrapping {} postcodes, result: {:?}",
                count,
                res
            );
            Some(())
        }
        Err(err) => {
            log::error!("Failed to write postcodes to geo positions, error: {}", err);
            None
        }
    }
}

// TODO [#47]: move these away
fn build_row_value(lat_index: usize, lon_index: usize, row: &StringRecord) -> String {
    format!(
        "{};{}",
        row.get(lat_index).unwrap(),
        row.get(lon_index).unwrap()
    )
}

// TODO [#47]: move these away
fn build_row_tuple(lat_index: usize, lon_index: usize, row: &StringRecord) -> (&str, &str) {
    (row.get(lon_index).unwrap(), row.get(lat_index).unwrap())
}

fn build_row_field(postcode_index: usize, row: &StringRecord) -> String {
    row.get(postcode_index)
        .unwrap()
        .to_string()
        .replace(" ", "")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_bulk_set() {
        let file_name = "./test.bulk.set.csv";

        let test_file = File::create(&file_name).expect("Unable to create ./test.csv");
        test_file.set_len(0).unwrap();
        let mut writer = csv::Writer::from_path(&file_name).expect("Issue reading test.csv");
        writer
            .write_record(&["TEST1", "0.0", "0.0"])
            .expect("Unable to write test record");
        let mut reader = csv::Reader::from_path(&file_name).expect("Issue reading test.csv");
        let set_count = bulk_set(&mut reader, POSTCODE_TABLE_NAME);
        fs::remove_file(&file_name).unwrap();
        assert_eq!(set_count, Some(()));
    }

    #[test]
    fn test_count() {
        set("TEST_TABLE_COUNT", "TEST", "TEST").unwrap();
        let table_count = count("TEST_TABLE_COUNT");
        assert_ne!(table_count, 0);
    }

    #[test]
    fn test_count_0() {
        let table_count = count("TEST");
        assert_eq!(table_count, 0);
    }

    #[test]
    fn test_set() {
        hdel("TEST_TABLE", "TEST");
        let result = set("TEST_TABLE", "TEST", "TEST").unwrap();
        assert_eq!(
            result,
            "Wrote TEST to table: TEST_TABLE with key TEST and result 1"
        );
    }

    #[test]
    fn test_set_json_none() {
        del("testjson");
        let result = set_json("testjson", None, "{\"item\":\"pass\"}");
        del("testjson");
        assert_ne!(result, None);
    }

    #[test]
    fn test_set_json() {
        del("testjson");
        let result = set_json("testjson", Some("."), "{\"item\":\"pass\"}");
        del("testjson");
        assert_ne!(result, None);
    }

    #[test]
    fn test_hdel() {
        let table_count = count("TEST_DEL_TABLE");
        println!("{}", table_count);
        if table_count == 0 {
            hdel("TEST_DEL_TABLE", "TEST");
        }
        set("TEST_DEL_TABLE", "TEST", "TEST").unwrap();
        hdel("TEST_DEL_TABLE", "TEST");
        let table_count = count("TEST_DEL_TABLE");
        assert_eq!(table_count, 0);
    }

    #[test]
    fn test_get() {
        set("TEST_GET_TABLE", "TEST", "TEST").unwrap();
        let get: String = get("TEST_GET_TABLE", "TEST").unwrap();
        assert_eq!(get, "TEST")
    }

    #[test]
    fn test_get_postcode() {}

    #[test]
    fn test_get_coordinates() {
        let key = "IMAGINARYPOSTCODE";
        hdel(POSTCODE_TABLE_NAME, key);
        set(POSTCODE_TABLE_NAME, key, "0.0;0.0").unwrap();
        let coordinates = get_coordinates(key).unwrap();
        assert_eq!(coordinates, "\"0.0;0.0\"")
    }

    #[test]
    fn test_get_redis_client() {
        assert!(get_redis_client().is_ok())
    }

    #[test]
    fn test_connect_and_query() {
        let result: Result<String, Error> =
            connect_and_query(|mut connection| Ok(connection.set("TEST_HOF", "TEST_HOF")?));
        assert!(result.is_ok());
    }
}
