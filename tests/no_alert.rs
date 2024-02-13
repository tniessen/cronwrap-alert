use cronwrap;

mod common;
use common::TestServer;

#[test]
fn test_no_alert() {
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
            command: "true".to_string(),
            args: vec![],
        }),
        0
    );

    let requests = server.stop();
    assert_eq!(requests.len(), 0);
}

#[test]
fn test_no_alert_ignored_exit_code() {
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
            ignore_exit_code: vec![1, 10, 100],
            program_name: None,
            command: "false".to_string(),
            args: vec![],
        }),
        0
    );

    let requests = server.stop();
    assert_eq!(requests.len(), 0);
}

#[test]
fn test_no_alert_slow() {
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
            command: "sleep".to_string(),
            args: vec!["1".to_string()],
        }),
        0
    );

    let requests = server.stop();
    assert_eq!(requests.len(), 0);
}
