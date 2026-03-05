#![allow(dead_code)]

use std::io::{Read, Write};

pub fn unique_cli_workspace_root(test_name: &str) -> std::path::PathBuf {
    let timestamp_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "specloom-cli-{test_name}-{}-{timestamp_nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(path.as_path()).expect("workspace should be created");
    path
}

pub fn specloom_binary_path() -> std::path::PathBuf {
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_specloom")
        .map(std::path::PathBuf::from)
        .filter(|path| path.is_file())
    {
        return path;
    }

    let compiled_path = std::path::PathBuf::from(env!("CARGO_BIN_EXE_specloom"));
    if compiled_path.is_file() {
        return compiled_path;
    }

    let mut derived = std::env::current_exe().expect("current test executable path should resolve");
    derived.pop();
    if derived.ends_with("deps") {
        derived.pop();
    }
    derived.push(format!("specloom{}", std::env::consts::EXE_SUFFIX));
    if derived.is_file() {
        return derived;
    }

    panic!(
        "failed to locate specloom binary: runtime env={:?}, compile-time={}, derived={}",
        std::env::var_os("CARGO_BIN_EXE_specloom"),
        env!("CARGO_BIN_EXE_specloom"),
        derived.display()
    );
}

pub fn specloom_command() -> std::process::Command {
    std::process::Command::new(specloom_binary_path())
}

pub fn cli_fixture_path(relative_path: &str) -> std::path::PathBuf {
    let relative_path = std::path::Path::new(relative_path);
    let mut candidates = Vec::new();

    if let Some(manifest_dir) = std::env::var_os("CARGO_MANIFEST_DIR").map(std::path::PathBuf::from)
    {
        candidates.push(manifest_dir.join(relative_path));
    }

    candidates.push(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path));

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("crates/cli").join(relative_path));
        candidates.push(current_dir.join(relative_path));
    }

    if let Ok(current_exe) = std::env::current_exe() {
        for ancestor in current_exe.ancestors() {
            candidates.push(ancestor.join("crates/cli").join(relative_path));
            candidates.push(ancestor.join(relative_path));
        }
    }

    if let Some(path) = candidates.into_iter().find(|candidate| candidate.is_file()) {
        return path;
    }

    panic!(
        "failed to locate fixture path {} via CARGO_MANIFEST_DIR/current_dir/current_exe fallbacks",
        relative_path.display()
    );
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
        let (mut stream, _) = listener
            .accept()
            .expect("image server should accept request");
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

pub fn seed_bundle_instruction_sources(workspace_root: &std::path::Path) {
    let skills_guide_path = workspace_root.join(".codex/SKILLS.md");
    if let Some(parent) = skills_guide_path.parent() {
        std::fs::create_dir_all(parent).expect("skills guide parent should be creatable");
    }
    std::fs::write(
        skills_guide_path.as_path(),
        r#"# Project Skills Guide

## Active Skills

1. `recognizing-layout`
Path: `.codex/skills/recognizing-layout/SKILL.md`
Use layout guidance.

2. `authoring-transform-plan`
Path: `.codex/skills/authoring-transform-plan/SKILL.md`
Use transform plan guidance.

## Usage Order
1. Example only
"#,
    )
    .expect("skills guide should be writable");

    let recognizing_layout_path = workspace_root.join(".codex/skills/recognizing-layout/SKILL.md");
    if let Some(parent) = recognizing_layout_path.parent() {
        std::fs::create_dir_all(parent).expect("recognizing-layout parent should be creatable");
    }
    std::fs::write(recognizing_layout_path.as_path(), "# recognizing layout")
        .expect("recognizing-layout skill should be writable");

    let authoring_transform_path =
        workspace_root.join(".codex/skills/authoring-transform-plan/SKILL.md");
    if let Some(parent) = authoring_transform_path.parent() {
        std::fs::create_dir_all(parent)
            .expect("authoring-transform-plan parent should be creatable");
    }
    std::fs::write(
        authoring_transform_path.as_path(),
        "# authoring transform plan",
    )
    .expect("authoring-transform-plan skill should be writable");

    let playbook_path = workspace_root.join("docs/agent-playbook.md");
    if let Some(parent) = playbook_path.parent() {
        std::fs::create_dir_all(parent).expect("playbook parent should be creatable");
    }
    std::fs::write(playbook_path.as_path(), "# agent playbook")
        .expect("agent playbook should be writable");

    let figma_ui_coder_path = workspace_root.join("docs/figma-ui-coder.md");
    std::fs::write(figma_ui_coder_path.as_path(), "# figma ui coder")
        .expect("figma ui coder doc should be writable");
}
