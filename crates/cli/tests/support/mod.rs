use std::io::{Read, Write};

pub fn unique_cli_workspace_root(test_name: &str) -> std::path::PathBuf {
    let timestamp_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "forge-cli-{test_name}-{}-{timestamp_nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(path.as_path()).expect("workspace should be created");
    path
}

pub fn start_live_api_server(
    image_url: &str,
) -> Result<(String, std::thread::JoinHandle<()>), std::io::Error> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let address = listener
        .local_addr()
        .expect("api server should expose local address");
    listener
        .set_nonblocking(true)
        .expect("api server should set nonblocking");
    let image_url = image_url.to_string();

    let server_thread = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut served = 0usize;
        while served < 2 && std::time::Instant::now() < deadline {
            let (mut stream, _) = match listener.accept() {
                Ok(conn) => conn,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(err) => panic!("api server should accept request: {err}"),
            };
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                .expect("api server should set read timeout");

            let mut request_bytes = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let bytes_read = stream
                    .read(&mut buffer)
                    .expect("api server should read request bytes");
                if bytes_read == 0 {
                    break;
                }
                request_bytes.extend_from_slice(&buffer[..bytes_read]);
                if request_bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }

            let request = String::from_utf8_lossy(&request_bytes);
            let first_line = request.lines().next().unwrap_or_default();

            let (status_line, body, content_type) =
                if first_line.starts_with("GET /v1/files/abc123/nodes?ids=123%3A456 HTTP/1.1") {
                    (
                        "200 OK",
                        r#"{
                            "nodes": {
                                "123:456": {
                                    "document": {
                                        "id": "123:456",
                                        "name": "Live Root",
                                        "type": "FRAME",
                                        "children": []
                                    }
                                }
                            }
                        }"#
                        .to_string(),
                        "application/json",
                    )
                } else if first_line
                    .starts_with("GET /v1/images/abc123?ids=123%3A456&format=png HTTP/1.1")
                {
                    (
                        "200 OK",
                        format!(r#"{{"images": {{"123:456": "{image_url}"}}}}"#),
                        "application/json",
                    )
                } else {
                    ("404 Not Found", "Not Found".to_string(), "text/plain")
                };

            let response = format!(
                "HTTP/1.1 {status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {content_length}\r\nConnection: close\r\n\r\n{body}",
                content_length = body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("api server should write response");
            stream.flush().expect("api server should flush response");
            served += 1;
        }
    });

    Ok((format!("http://{address}"), server_thread))
}

pub fn start_single_binary_response_server(
    expected_path: &str,
    body: &[u8],
) -> Result<(String, std::thread::JoinHandle<()>), std::io::Error> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let address = listener
        .local_addr()
        .expect("image server should expose local address");
    let expected_path = expected_path.to_string();
    let body = body.to_vec();

    let server_thread = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("image server should accept request");
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .expect("image server should set read timeout");

        let mut request_bytes = Vec::new();
        let mut buffer = [0_u8; 4096];
        loop {
            let bytes_read = stream
                .read(&mut buffer)
                .expect("image server should read request bytes");
            if bytes_read == 0 {
                break;
            }
            request_bytes.extend_from_slice(&buffer[..bytes_read]);
            if request_bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }

        let request = String::from_utf8_lossy(&request_bytes);
        let first_line = request.lines().next().unwrap_or_default();
        let status_line = if first_line.starts_with(format!("GET {expected_path} ").as_str()) {
            "200 OK"
        } else {
            "404 Not Found"
        };
        let response_body = if status_line == "200 OK" {
            body.clone()
        } else {
            b"not found".to_vec()
        };

        let mut response_head = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: image/png\r\nContent-Length: {content_length}\r\nConnection: close\r\n\r\n",
            content_length = response_body.len()
        )
        .into_bytes();
        response_head.extend_from_slice(response_body.as_slice());

        stream
            .write_all(response_head.as_slice())
            .expect("image server should write response");
        stream.flush().expect("image server should flush response");
    });

    Ok((format!("http://{address}"), server_thread))
}
