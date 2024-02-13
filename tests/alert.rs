use cronwrap;

mod common;
use common::TestServer;

#[test]
fn test_alert() {
    let server = TestServer::start();

    assert_eq!(
        cronwrap::main(cronwrap::Args {
            alert_channel: Some("foo".to_string()),
            alert_category: None,
            alert_origin: None,
            no_alert_origin: false,
            alert_endpoint: Some(server.endpoint_url()),
            alert_token: Some(server.token()),
            max_output_capture: 65536,
            ignore_exit_code: vec![],
            program_name: None,
            command: "false".to_string(),
            args: vec![],
        }),
        1
    );

    let requests = server.stop();
    println!("{:?}", requests[0]);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].as_object().unwrap().len(), 6);
    assert_eq!(requests[0]["channel"], "foo");
    assert_eq!(requests[0]["category"], serde_json::Value::Null);
    assert_eq!(
        requests[0]["origin"],
        gethostname::gethostname().to_string_lossy().into_owned()
    );
    assert_eq!(requests[0]["contentType"], "text/plain");
    assert_eq!(requests[0]["subject"], "Scheduled execution failed: false");
}

#[test]
fn test_alert_with_ignored_exit_code() {
    let server = TestServer::start();

    // The command "false" returns 1, which is not among the ignored exit codes.
    assert_eq!(
        cronwrap::main(cronwrap::Args {
            alert_channel: Some("foo".to_string()),
            alert_category: None,
            alert_origin: None,
            no_alert_origin: false,
            alert_endpoint: Some(server.endpoint_url()),
            alert_token: Some(server.token()),
            max_output_capture: 65536,
            ignore_exit_code: vec![2, 3, 4, 10],
            program_name: None,
            command: "false".to_string(),
            args: vec![],
        }),
        1
    );

    let requests = server.stop();
    println!("{:?}", requests[0]);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].as_object().unwrap().len(), 6);
    assert_eq!(requests[0]["channel"], "foo");
    assert_eq!(requests[0]["category"], serde_json::Value::Null);
    assert_eq!(
        requests[0]["origin"],
        gethostname::gethostname().to_string_lossy().into_owned()
    );
    assert_eq!(requests[0]["contentType"], "text/plain");
    assert_eq!(requests[0]["subject"], "Scheduled execution failed: false");
}

#[test]
fn test_alert_with_category() {
    let server = TestServer::start();

    assert_eq!(
        cronwrap::main(cronwrap::Args {
            alert_channel: Some("foo".to_string()),
            alert_category: Some("bar".to_string()),
            alert_origin: None,
            no_alert_origin: false,
            alert_endpoint: Some(server.endpoint_url()),
            alert_token: Some(server.token()),
            max_output_capture: 65536,
            ignore_exit_code: vec![],
            program_name: None,
            command: "false".to_string(),
            args: vec![],
        }),
        1
    );

    let requests = server.stop();
    println!("{:?}", requests[0]);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].as_object().unwrap().len(), 6);
    assert_eq!(requests[0]["channel"], "foo");
    assert_eq!(requests[0]["category"], "bar");
    assert_eq!(
        requests[0]["origin"],
        gethostname::gethostname().to_string_lossy().into_owned()
    );
    assert_eq!(requests[0]["contentType"], "text/plain");
    assert_eq!(requests[0]["subject"], "Scheduled execution failed: false");
}

#[test]
fn test_alert_with_origin() {
    let server = TestServer::start();

    assert_eq!(
        cronwrap::main(cronwrap::Args {
            alert_channel: Some("foo".to_string()),
            alert_category: None,
            alert_origin: Some("server-1234".to_string()),
            no_alert_origin: false,
            alert_endpoint: Some(server.endpoint_url()),
            alert_token: Some(server.token()),
            max_output_capture: 65536,
            ignore_exit_code: vec![],
            program_name: None,
            command: "false".to_string(),
            args: vec![],
        }),
        1
    );

    let requests = server.stop();
    println!("{:?}", requests[0]);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].as_object().unwrap().len(), 6);
    assert_eq!(requests[0]["channel"], "foo");
    assert_eq!(requests[0]["category"], serde_json::Value::Null);
    assert_eq!(requests[0]["origin"], "server-1234");
    assert_eq!(requests[0]["contentType"], "text/plain");
    assert_eq!(requests[0]["subject"], "Scheduled execution failed: false");
}

#[test]
fn test_alert_without_origin() {
    let server = TestServer::start();

    assert_eq!(
        cronwrap::main(cronwrap::Args {
            alert_channel: Some("foo".to_string()),
            alert_category: None,
            alert_origin: None,
            no_alert_origin: true,
            alert_endpoint: Some(server.endpoint_url()),
            alert_token: Some(server.token()),
            max_output_capture: 65536,
            ignore_exit_code: vec![],
            program_name: None,
            command: "false".to_string(),
            args: vec![],
        }),
        1
    );

    let requests = server.stop();
    println!("{:?}", requests[0]);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].as_object().unwrap().len(), 6);
    assert_eq!(requests[0]["channel"], "foo");
    assert_eq!(requests[0]["category"], serde_json::Value::Null);
    assert_eq!(requests[0]["origin"], serde_json::Value::Null);
    assert_eq!(requests[0]["contentType"], "text/plain");
    assert_eq!(requests[0]["subject"], "Scheduled execution failed: false");
}
