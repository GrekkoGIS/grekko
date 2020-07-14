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

fn build_row_value(lat_index: usize, lon_index: usize, row: &StringRecord) -> String {
    format!(
        "{};{}",
        row.get(lat_index).unwrap(),
        row.get(lon_index).unwrap()
    )
}

pub(crate) fn build_row_tuple(
    lat_index: usize,
    lon_index: usize,
    row: &StringRecord,
) -> (&str, &str) {
    (row.get(lon_index).unwrap(), row.get(lat_index).unwrap())
}

pub(crate) fn build_row_field(postcode_index: usize, row: &StringRecord) -> String {
    row.get(postcode_index)
        .unwrap()
        .to_string()
        .replace(" ", "")
}
