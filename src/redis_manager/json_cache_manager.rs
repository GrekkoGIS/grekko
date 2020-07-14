use csv::{Reader, StringRecord};
use failure::{Error, ResultExt};
use redis::geo::Coord;
use redis::{Client, Cmd, Commands, Connection, RedisError, RedisResult, Value};
use serde::de::DeserializeOwned;
use serde::export::fmt::Display;
use serde::Serialize;

pub trait JsonCacheManager {
    fn set_json<T: Serialize + Display>(
        &self,
        key: &str,
        path: Option<&str>,
        value: T,
    ) -> Option<String>;

    fn get_json<T: DeserializeOwned>(&self, key: &str, path: Option<&str>) -> Result<T, Error>;

    fn append_json<T: Serialize>(&self, key: &str, path: &str, value: T) -> Option<String>;
}
