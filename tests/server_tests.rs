#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::{
        io::{BufRead, BufReader, Write},
        net::{TcpListener, TcpStream},
        sync::mpsc,
        thread,
        time::Duration,
    };

    struct TestServer {
        port: u16,
        _handle: thread::JoinHandle<()>, // Using _ to indicate we keep it for lifetime management
    }

    impl TestServer {
        fn start() -> Self {
            let (tx, rx) = mpsc::channel();

            // Start server in separate thread
            let handle = thread::spawn(move || {
                // Start TCP server on random port
                let listener = TcpListener::bind("127.0.0.1:0").unwrap();
                let port = listener.local_addr().unwrap().port();
                tx.send(port).unwrap();

                println!("Test server listening on port {}", port);

                // Accept and handle one connection
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let mut reader = BufReader::new(stream.try_clone().unwrap());
                        let mut line = String::new();

                        while let Ok(n) = reader.read_line(&mut line) {
                            if n == 0 {
                                break;
                            } // Connection closed

                            println!("Server received: {}", line.trim());

                            // Parse the command and send appropriate response
                            let response = match serde_json::from_str::<serde_json::Value>(&line) {
                                Ok(cmd) => match cmd["type"].as_str() {
                                    Some("Initialize") => json!({
                                        "status": "ready",
                                        "aircraft": [{
                                            "name": "aircraft_0",
                                            "config": {},
                                            "action_space": {},
                                            "observation_space": {}
                                        }]
                                    }),
                                    Some("Step") => json!({
                                        "obs": {
                                            "aircraft_0": {
                                                "x": 0.0,
                                                "y": 0.0,
                                                "heading": 0.0
                                            }
                                        }
                                    }),
                                    Some("Reset") => json!({
                                        "obs": {
                                            "aircraft_0": {
                                                "x": 0.0,
                                                "y": 0.0,
                                                "heading": 0.0
                                            }
                                        },
                                        "reward": 0.0,
                                        "terminated": false,
                                        "truncated": false,
                                        "info": {}
                                    }),
                                    _ => json!({"error": "Unknown command"}),
                                },
                                Err(e) => json!({"error": format!("Invalid JSON: {}", e)}),
                            };

                            // Send response
                            let response_str = serde_json::to_string(&response).unwrap() + "\n";
                            stream.write_all(response_str.as_bytes()).unwrap();
                            stream.flush().unwrap();

                            line.clear();
                        }
                    }
                    Err(e) => println!("Error accepting connection: {}", e),
                }
            });

            // Wait for server to start and get port
            let port = rx.recv().unwrap();

            // Give the server a moment to fully start
            thread::sleep(Duration::from_millis(100));

            TestServer {
                port,
                _handle: handle,
            }
        }

        fn port(&self) -> u16 {
            self.port
        }
    }

    struct TestClient {
        stream: TcpStream,
        reader: BufReader<TcpStream>,
    }

    impl TestClient {
        fn connect(port: u16) -> std::io::Result<Self> {
            let stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
            stream.set_read_timeout(Some(Duration::from_secs(2)))?;
            stream.set_write_timeout(Some(Duration::from_secs(2)))?;
            let reader = BufReader::new(stream.try_clone()?);

            Ok(Self { stream, reader })
        }

        fn send_raw(&mut self, msg: &str) -> std::io::Result<String> {
            let msg = msg.to_string() + "\n";
            self.stream.write_all(msg.as_bytes())?;
            self.stream.flush()?;

            let mut response = String::new();
            self.reader.read_line(&mut response)?;
            Ok(response)
        }
    }

    #[test]
    fn test_protocol() -> std::io::Result<()> {
        // Start test server
        let server = TestServer::start();
        println!("Test server started on port {}", server.port());

        // Connect client
        let mut client = TestClient::connect(server.port())?;

        // Test initialization
        println!("Sending initialization command...");
        let init_response = client.send_raw(
            r#"{
            "type": "Initialize",
            "config": {
                "seed": 42,
                "max_episode_steps": 1000,
                "aircraft_config": [{
                    "type": "dubins",
                    "action_type": "Continuous",
                    "observation_type": "Continuous"
                }]
            }
        }"#,
        )?;

        println!("Received init response: {}", init_response);
        let init_json: serde_json::Value =
            serde_json::from_str(&init_response).expect("Failed to parse initialization response");
        assert!(init_json.get("status").is_some(), "Missing status field");
        assert!(
            init_json.get("aircraft").is_some(),
            "Missing aircraft field"
        );

        // Test step command
        let step_response = client.send_raw(
            r#"{
            "type": "Step",
            "actions": {
                "aircraft_0": [0.0, 0.0, 0.0]
            }
        }"#,
        )?;

        let step_json: serde_json::Value =
            serde_json::from_str(&step_response).expect("Failed to parse step response");
        assert!(step_json.is_object(), "Step response should be an object");

        Ok(())
    }

    #[test]
    fn test_error_handling() -> std::io::Result<()> {
        let server = TestServer::start();
        let mut client = TestClient::connect(server.port())?;

        // Test malformed JSON
        let response = client.send_raw("not json")?;
        let json: serde_json::Value =
            serde_json::from_str(&response).expect("Failed to parse error response");
        assert!(json.get("error").is_some());

        // Test missing fields
        let response = client.send_raw(r#"{"type": "Step"}"#)?;
        let json: serde_json::Value =
            serde_json::from_str(&response).expect("Failed to parse error response");
        assert!(json.get("error").is_some());

        Ok(())
    }
}
