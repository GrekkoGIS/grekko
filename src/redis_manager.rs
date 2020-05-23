use std::fs::File;
use std::rc::Rc;
use std::sync::Arc;

use cached::SizedCache;
use csv::{Reader, StringRecordIter, StringRecordsIter};
use redis::{AsyncCommands, Client, Commands, RedisResult};

// cached_key! {
//     REDIS_CLIENT: SizedCache<String, RedisResult<Client>> = SizedCache::with_size(1);
//     Key = String::from("redis_client");
//
// }

fn get_redis_client() -> RedisResult<Client> {
    redis::Client::open("redis://127.0.0.1/")
}

pub async fn get(query: &str) -> Option<String> {
    let client: Client = get_redis_client().ok()?;
    let mut con = client.get_async_connection().await.ok()?;

    con.get(query).await.ok()?
}

pub async fn bulk_set(reader: &mut Reader<File>) {
    let records = reader.records();
    let client: Client = get_redis_client().unwrap();
    let mut con = client.get_async_connection().await.unwrap();

    let postcode_index = 0;
    let lat_index = 1;
    let lon_index = 2;

    let mut pipeline = redis::pipe();
    records.for_each(|row| {
        let row = &row.unwrap();
        &pipeline
            .cmd("SET")
            .arg(row.get(postcode_index).unwrap().to_string().replace(" ", ""))
            .arg(format!("{};{}", row.get(lat_index).unwrap(), row.get(lon_index).unwrap()))
            .ignore();
    }
    );

    let result: RedisResult<String> = pipeline.query_async(&mut con).await;
    println!("{}", result.unwrap());
}