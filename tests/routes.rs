extern crate grekko;

use warp::reply::Reply;

#[tokio::test]
async fn test_geocoding_forward() {
    let result = grekko::receive_and_search_postcode(57.099011, -2.252854).await.unwrap().into_response().status();
    let expected = warp::reply::json(&"AB1 0AJ");
    assert_eq!(&result.as_str(), &expected.into_response().status().as_str())
}

#[tokio::test]
async fn test_geocoding_reverse() {
    let result = grekko::receive_and_search_coordinates("AB1-0AJ".to_string()).await.unwrap().into_response().status();
    let expected = warp::reply::json(&vec![57.099011, -2.252854]);
    assert_eq!(&result.as_str(), &expected.into_response().status().as_str())
}