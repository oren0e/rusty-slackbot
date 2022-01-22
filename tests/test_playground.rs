use httpmock::prelude::*;
use rstest::*;
use rusty_slackbot::playground::{PlaygroundRequest, PlaygroundResponse, ShareResponse};
use serde_json::json;
use serde_json::Value;
use std::fs;

#[fixture]
fn good_eval_response() -> Value {
    let s = fs::read_to_string("tests/data/pg_eval_response_good.json")
        .expect("Error: good_eval_response read json file failed in tests");
    serde_json::from_str(&s).expect("Failed parsing json in good_eval_response in tests")
}

#[fixture]
fn bad_eval_response() -> Value {
    let s = fs::read_to_string("tests/data/pg_eval_response_bad.json")
        .expect("Error: bad_eval_response read json file failed in tests");
    serde_json::from_str(&s).expect("Failed parsing json in bad_eval_response in tests")
}

#[fixture]
fn good_code() -> String {
    "println!(\"Hello World\");".to_owned()
}

#[fixture]
fn bad_code() -> String {
    "println!(\"Hello World\"".to_owned()
}

#[fixture]
fn share_link_response() -> Value {
    let s = fs::read_to_string("tests/data/pg_response_share_link.json")
        .expect("Error: share_link_response read json file failed in tests");
    serde_json::from_str(&s).expect("Failed parsing json in share_link_response in tests")
}

#[rstest]
#[case(good_code(), good_eval_response())]
#[case(bad_code(), bad_eval_response())]
#[tokio::test]
async fn test_eval_execute(#[case] code: String, #[case] raw_response: Value) {
    let payload = json!(
                {
        "channel": "stable",
        "mode": "debug",
        "edition": "2021",
        "crateType": "bin",
        "tests": false,
        "code": format!("fn main() {{{}}}", code),
        "backtrace": false
    }
                );
    let expected_response: PlaygroundResponse = serde_json::from_value(raw_response.clone())
        .expect("Failed to convert from value to PlaygroundResponse in test_eval_execute");
    let server = MockServer::start_async().await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/execute")
            .header("Content-Type", "application/json")
            .json_body(payload);
        then.status(200).json_body(raw_response);
    });
    let request = PlaygroundRequest::new_eval(code).escape_html();
    let response = request.execute(&server.base_url()).await.unwrap();

    mock.assert();
    assert_eq!(response.status_code, "200".to_owned());
    assert_eq!(
        response.playground_response.success,
        expected_response.success
    );
    assert_eq!(
        response.playground_response.stdout,
        expected_response.stdout
    );
    assert_eq!(
        response.playground_response.stderr,
        expected_response.stderr
    );
}

#[rstest]
#[tokio::test]
async fn test_create_share_link(good_code: String, share_link_response: Value) {
    let payload = json!({ "code": format!("fn main() {{{}}}", good_code) });
    let parsed_response: ShareResponse = serde_json::from_value(share_link_response.clone())
        .expect("Failed to convert from Value to ShareResponse in test_create_share_link");
    let request = PlaygroundRequest::new_eval(good_code).escape_html();
    let server = MockServer::start_async().await;
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/meta/gist/")
            .header("Content-Type", "application/json")
            .json_body(payload);
        then.status(200).json_body(share_link_response);
    });
    let response = request.create_share_link(&server.base_url()).await.unwrap();

    mock.assert();
    assert_eq!(
        format!(
            "https://play.rust-lang.org/?version={}&mode=debug&edition={}&gist={}",
            request.get_channel(),
            request.get_edition(),
            parsed_response.id
        ),
        response
    );
}
