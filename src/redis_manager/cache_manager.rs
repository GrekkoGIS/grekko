
use failure::{Error, ResultExt};
use redis::{Client, Commands, Connection, RedisResult};
use serde::de::DeserializeOwned;
use serde::export::fmt::Display;
use serde::Serialize;

pub trait CacheManager {
    fn connect_and_query<F, T>(&self, client: &Client, mut action: F) -> Result<T, Error>
    where
        F: FnMut(Connection) -> Result<T, Error>,
    {
        let con = client.get_connection().with_context(|err| {
            format!("Failed to get a connection, category `{}`", err.category())
        })?;
        action(con)
    }

    fn hget<T: DeserializeOwned>(
        &self,
        client: &Client,
        table: &str,
        key: &str,
    ) -> Result<T, Error> {
        let result: Result<String, Error> = self.connect_and_query(&client, |mut connection| {
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

    fn hdel(&self, client: &Client, table: &str, key: &str) -> Result<String, Error> {
        self.connect_and_query(&client, |mut connection| Ok(connection.hdel(table, key)?))
    }

    fn del(&self, client: &Client, key: &str) -> Result<String, Error> {
        self.connect_and_query(&client, |mut connection| Ok(connection.del(key)?))
    }

    fn hset<T: Serialize + Display>(
        &self,
        client: &Client,
        table: &str,
        key: &str,
        value: T,
    ) -> Option<String> {
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

    fn count(&self, client: &Client, table: &str) -> i32 {
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
}
