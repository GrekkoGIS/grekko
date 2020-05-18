extern crate grekko;

use warp::reply::Reply;

#[tokio::test]
async fn test_geocoding_forward() {
    let result = grekko::receive_and_search_postcode(57.099011, -2.252854)
        .await
        .unwrap();
    let result = result.into_response();
    let result_status = result.status();
    let result_status = result_status.as_str();

    let expected = warp::reply::json(&"AB1 0AJ").into_response();
    let expected_status = expected.status();
    let expected_status = expected_status.as_str();

    assert_eq!(&result_status, &expected_status)
}

#[tokio::test]
async fn test_geocoding_reverse() {
    let result = grekko::receive_and_search_coordinates("AB1-0AJ".to_string())
        .await
        .unwrap()
        .into_response()
        .status();
    let expected = warp::reply::json(&vec![57.099011, -2.252854]);
    assert_eq!(
        &result.as_str(),
        &expected.into_response().status().as_str()
    )
}
