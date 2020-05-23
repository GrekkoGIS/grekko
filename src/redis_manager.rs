use std::fs::File;

use csv::{Reader};
use redis::{AsyncCommands, Client, RedisResult};

fn get_redis_client() -> RedisResult<Client> {
    redis::Client::open("redis://127.0.0.1/")
}

pub async fn get(query: &str) -> Option<String> {
    let client: Client = get_redis_client().ok()?;
    let mut con = client.get_async_connection().await.ok()?;

    con.get(query).await.ok()?
}

pub async fn count(table: &str) -> i32 {
    let client: Client = get_redis_client().unwrap();
    let mut con = client.get_async_connection().await.unwrap();

    con.hlen(table).await.unwrap()
}

pub async fn bulk_set(reader: &mut Reader<File>) {
    let records = reader.records();
    let client: Client = get_redis_client().unwrap();
    let mut con = client.get_async_connection().await.unwrap();

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
                format!("{};{}", row.get(lat_index).unwrap(), row.get(lon_index).unwrap())
            )
            .ignore();
    }
    );

    let result: RedisResult<i32> = pipeline.query_async(&mut con).await;

    println!("Finished bootstrapping {} postcodes, result: {:?}", count, result);
}