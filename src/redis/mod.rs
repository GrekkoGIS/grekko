use cached::SizedCache;
use redis;

cached_key! {
    REDIS_CLIENT: SizedCache<String, RedisResult<Client>> = SizedCache::with_size(1);
    Key = String::from("redis_client");
    fn get_redis_client() -> RedisResult<Client> = {
        let redis = redis::Client::open("redis://127.0.0.1/");
        let mut con = client.get_connection()?;
        con
    }
}

pub fn get(query: &str) {
    let client = get_redis_client();
}
