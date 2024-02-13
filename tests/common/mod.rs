use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{spawn, JoinHandle};
use std::time::Duration;

use serde_json::{json, Value};

pub struct TestServer {
    server: Arc<tiny_http::Server>,
    listen: Arc<AtomicBool>,
    listener: JoinHandle<Vec<Value>>,
}

fn token_from_arc(arc: &Arc<tiny_http::Server>) -> String {
    format!("{:x}", Arc::as_ptr(arc) as usize)
}

impl TestServer {
    pub fn start() -> Self {
        let listen = Arc::new(AtomicBool::new(true));
        let server = Arc::new(tiny_http::Server::http("0.0.0.0:0").unwrap());

        TestServer {
            server: server.clone(),
            listen: listen.clone(),
            listener: spawn(move || -> Vec<Value> {
                // Every request must send the authorization token.
                let bearer = format!("Bearer {}", token_from_arc(&server));
                eprintln!("Authorization: {}", bearer);

                let mut requests: Vec<Value> = vec![];
                while listen.load(Ordering::Relaxed) {
                    let maybe_request = server.recv_timeout(Duration::from_millis(100)).unwrap();
                    if let Some(mut request) = maybe_request {
                        assert_eq!(request.url(), "/v1/post");
                        assert_eq!(request.method(), &tiny_http::Method::Post);
                        assert_eq!(
                            request
                                .headers()
                                .iter()
                                .find(|h| h.field.equiv(&"Content-Type"))
                                .map(|h| -> &str { h.value.as_ref() })
                                .unwrap(),
                            "application/json"
                        );
                        assert_eq!(
                            request
                                .headers()
                                .iter()
                                .find(|h| h.field.equiv(&"Authorization"))
                                .map(|h| -> &str { h.value.as_ref() })
                                .unwrap(),
                            bearer
                        );

                        assert_ne!(request.body_length().unwrap(), 0);
                        let mut body = String::new();
                        request.as_reader().read_to_string(&mut body).unwrap();
                        let json: Value = body.parse().unwrap();
                        assert!(json.is_object());
                        requests.push(json);

                        request
                            .respond(tiny_http::Response::from_data(
                                json!({
                                    "key": format!("alert/{}", requests.len()),
                                })
                                .to_string(),
                            ))
                            .unwrap();
                    }
                }
                requests
            }),
        }
    }

    fn host(&self) -> String {
        let port = self.server.server_addr().to_ip().unwrap().port();
        format!("127.0.0.1:{}", port)
    }

    pub fn endpoint_url(&self) -> String {
        format!("http://{}/v1/post", self.host())
    }

    pub fn token(&self) -> String {
        token_from_arc(&self.server)
    }

    pub fn stop(self) -> Vec<Value> {
        self.listen.store(false, Ordering::Relaxed);
        self.listener.join().unwrap()
    }
}
