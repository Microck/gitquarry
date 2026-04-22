use std::fs;
use std::io::Write;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use tempfile::TempDir;
use tiny_http::{Header, Response, Server, StatusCode};

fn start_fixture_server() -> (String, mpsc::Sender<()>, Arc<AtomicUsize>) {
    let server = Server::http("127.0.0.1:0").unwrap();
    let address = format!("http://{}/api/v3", server.server_addr());
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let request_count = Arc::new(AtomicUsize::new(0));
    let request_count_for_thread = Arc::clone(&request_count);

    thread::spawn(move || serve(server, stop_rx, request_count_for_thread));
    (address, stop_tx, request_count)
}

fn serve(server: Server, stop_rx: Receiver<()>, request_count: Arc<AtomicUsize>) {
    loop {
        if stop_rx.try_recv().is_ok() {
            break;
        }

        match server.recv_timeout(Duration::from_millis(50)) {
            Ok(Some(request)) => {
                request_count.fetch_add(1, Ordering::SeqCst);
                let url = request.url().to_string();
                let response = match url.as_str() {
                    path if path.starts_with("/api/v3/search/repositories")
                        && path.contains("q=empty+repo") =>
                    {
                        json_response(
                            200,
                            r#"{
                              "total_count": 1,
                              "items": [
                                {
                                  "name": "empty",
                                  "full_name": "example/empty",
                                  "html_url": "https://example.test/example/empty",
                                  "description": "An empty repository fixture",
                                  "stargazers_count": 1,
                                  "forks_count": 0,
                                  "language": "Rust",
                                  "topics": [],
                                  "license": null,
                                  "created_at": "2024-01-10T00:00:00Z",
                                  "updated_at": "2026-04-20T00:00:00Z",
                                  "pushed_at": "2026-04-19T00:00:00Z",
                                  "archived": false,
                                  "is_template": false,
                                  "fork": false,
                                  "open_issues_count": 0,
                                  "owner": { "login": "example" }
                                }
                              ]
                            }"#,
                        )
                    }
                    path if path.starts_with("/api/v3/search/repositories") => json_response(
                        200,
                        r#"{
                          "total_count": 1,
                          "items": [
                            {
                              "name": "rocket",
                              "full_name": "example/rocket",
                              "html_url": "https://example.test/example/rocket",
                              "description": "A fast Rust CLI search tool",
                              "stargazers_count": 420,
                              "forks_count": 32,
                              "language": "Rust",
                              "topics": ["cli", "search"],
                              "license": { "key": "mit", "name": "MIT License", "spdx_id": "MIT" },
                              "created_at": "2024-01-10T00:00:00Z",
                              "updated_at": "2026-04-20T00:00:00Z",
                              "pushed_at": "2026-04-19T00:00:00Z",
                              "archived": false,
                              "is_template": false,
                              "fork": false,
                              "open_issues_count": 4,
                              "owner": { "login": "example" }
                            }
                          ]
                        }"#,
                    ),
                    "/api/v3/repos/example/rocket" => json_response(
                        200,
                        r#"{
                          "name": "rocket",
                          "full_name": "example/rocket",
                          "html_url": "https://example.test/example/rocket",
                          "description": "A fast Rust CLI search tool",
                          "stargazers_count": 420,
                          "forks_count": 32,
                          "language": "Rust",
                          "topics": ["cli", "search", "readme"],
                          "license": { "key": "mit", "name": "MIT License", "spdx_id": "MIT" },
                          "created_at": "2024-01-10T00:00:00Z",
                          "updated_at": "2026-04-20T00:00:00Z",
                          "pushed_at": "2026-04-19T00:00:00Z",
                          "archived": false,
                          "is_template": false,
                          "fork": false,
                          "open_issues_count": 4,
                          "owner": { "login": "example" }
                        }"#,
                    ),
                    "/api/v3/repos/example/rocket/contributors?per_page=1&anon=1" => {
                        json_response(200, r#"[{"login":"a"},{"login":"b"},{"login":"c"}]"#)
                    }
                    "/api/v3/repos/example/big" => json_response(
                        200,
                        r#"{
                          "name": "big",
                          "full_name": "example/big",
                          "html_url": "https://example.test/example/big",
                          "description": "A bigger Rust CLI search tool",
                          "stargazers_count": 999,
                          "forks_count": 88,
                          "language": "Rust",
                          "topics": ["cli", "search"],
                          "license": { "key": "mit", "name": "MIT License", "spdx_id": "MIT" },
                          "created_at": "2024-01-10T00:00:00Z",
                          "updated_at": "2026-04-20T00:00:00Z",
                          "pushed_at": "2026-04-19T00:00:00Z",
                          "archived": false,
                          "is_template": false,
                          "fork": false,
                          "open_issues_count": 4,
                          "owner": { "login": "example" }
                        }"#,
                    ),
                    "/api/v3/repos/example/big/contributors?per_page=1&anon=1" => {
                        linked_json_response(
                            200,
                            r#"[{"login":"a"}]"#,
                            "<http://127.0.0.1/api/v3/repos/example/big/contributors?per_page=1&anon=1&page=2>; rel=\"next\", <http://127.0.0.1/api/v3/repos/example/big/contributors?per_page=1&anon=1&page=37>; rel=\"last\"",
                        )
                    }
                    "/api/v3/repos/example/empty" => json_response(
                        200,
                        r#"{
                          "name": "empty",
                          "full_name": "example/empty",
                          "html_url": "https://example.test/example/empty",
                          "description": "An empty repository fixture",
                          "stargazers_count": 1,
                          "forks_count": 0,
                          "language": "Rust",
                          "topics": [],
                          "license": null,
                          "created_at": "2024-01-10T00:00:00Z",
                          "updated_at": "2026-04-20T00:00:00Z",
                          "pushed_at": "2026-04-19T00:00:00Z",
                          "archived": false,
                          "is_template": false,
                          "fork": false,
                          "open_issues_count": 0,
                          "owner": { "login": "example" }
                        }"#,
                    ),
                    "/api/v3/repos/example/empty/contributors?per_page=1&anon=1" => {
                        raw_response(204, "")
                    }
                    "/api/v3/repos/example/huge" => json_response(
                        200,
                        r#"{
                          "name": "huge",
                          "full_name": "example/huge",
                          "html_url": "https://example.test/example/huge",
                          "description": "A huge Rust CLI search tool",
                          "stargazers_count": 1000,
                          "forks_count": 99,
                          "language": "Rust",
                          "topics": ["cli", "search"],
                          "license": { "key": "mit", "name": "MIT License", "spdx_id": "MIT" },
                          "created_at": "2024-01-10T00:00:00Z",
                          "updated_at": "2026-04-20T00:00:00Z",
                          "pushed_at": "2026-04-19T00:00:00Z",
                          "archived": false,
                          "is_template": false,
                          "fork": false,
                          "open_issues_count": 4,
                          "owner": { "login": "example" }
                        }"#,
                    ),
                    "/api/v3/repos/example/huge/contributors?per_page=1&anon=1" => json_response(
                        403,
                        r#"{
                          "message": "The history or contributor list is too large to list contributors for this repository via the API."
                        }"#,
                    ),
                    "/api/v3/repos/example/rocket/releases/latest" => json_response(
                        200,
                        r#"{
                          "tag_name": "v1.2.3",
                          "name": "v1.2.3",
                          "published_at": "2026-04-18T00:00:00Z",
                          "html_url": "https://example.test/example/rocket/releases/v1.2.3"
                        }"#,
                    ),
                    "/api/v3/repos/example/rocket/readme" => {
                        raw_response(200, "# Rocket\n\nRust CLI README fixture.\n")
                    }
                    "/api/v3/user" => json_response(200, r#"{"login":"fixture-user"}"#),
                    _ => raw_response(404, ""),
                };

                request.respond(response).unwrap();
            }
            Ok(None) => continue,
            Err(_) => break,
        }
    }
}

fn json_response(status: u16, body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let header = Header::from_bytes("Content-Type", "application/json").unwrap();
    Response::from_string(body.to_string())
        .with_status_code(StatusCode(status))
        .with_header(header)
}

fn raw_response(status: u16, body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let header = Header::from_bytes("Content-Type", "text/plain; charset=utf-8").unwrap();
    Response::from_string(body.to_string())
        .with_status_code(StatusCode(status))
        .with_header(header)
}

fn linked_json_response(status: u16, body: &str, link: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let content_type = Header::from_bytes("Content-Type", "application/json").unwrap();
    let link_header = Header::from_bytes("Link", link).unwrap();
    Response::from_string(body.to_string())
        .with_status_code(StatusCode(status))
        .with_header(content_type)
        .with_header(link_header)
}

fn base_command(temp: &TempDir, host: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_gitquarry"));
    command.env("GITQUARRY_TOKEN", "fixture-token");
    command.env("GITQUARRY_CONFIG_DIR", temp.path());
    command.arg("--host").arg(host);
    command
}

fn bare_command(temp: &TempDir, host: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_gitquarry"));
    command.env("GITQUARRY_CONFIG_DIR", temp.path());
    command.arg("--host").arg(host);
    command
}

fn host_key(host: &str) -> String {
    host.trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches("/api/v3")
        .to_string()
}

#[test]
fn search_json_includes_readme_enrichment() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = base_command(&temp, &host)
        .args([
            "search",
            "rust cli",
            "--mode",
            "discover",
            "--format",
            "json",
            "--readme",
            "--rank",
            "blended",
            "--concurrency",
            "2",
        ])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(payload["items"][0]["full_name"], "example/rocket");
    assert_eq!(
        payload["items"][0]["readme"],
        "# Rocket\n\nRust CLI README fixture.\n"
    );
    assert_eq!(payload["items"][0]["latest_release"]["tag_name"], "v1.2.3");
    assert_eq!(payload["items"][0]["contributor_count"], 3);
}

#[test]
fn inspect_json_returns_metadata_and_readme() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = base_command(&temp, &host)
        .args(["inspect", "example/rocket", "--readme", "--format", "json"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(payload["repository"]["full_name"], "example/rocket");
    assert_eq!(
        payload["repository"]["readme"],
        "# Rocket\n\nRust CLI README fixture.\n"
    );
    assert_eq!(
        payload["repository"]["latest_release"]["tag_name"],
        "v1.2.3"
    );
}

#[test]
fn auth_status_reports_saved_pat_only_when_env_override_active() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = base_command(&temp, &host)
        .args(["auth", "status"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("no saved token"));
    assert!(stdout.contains("EnvGlobal"));
}

#[test]
fn auth_status_prefers_env_override_even_when_saved_token_exists() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();
    let credentials = format!("[hosts]\n\"{}\" = \"saved-token\"\n", host_key(&host));
    fs::write(temp.path().join("credentials.toml"), credentials).unwrap();

    let output = base_command(&temp, &host)
        .env("GITQUARRY_ALLOW_INSECURE_STORAGE", "1")
        .args(["auth", "status"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("environment override active"));
    assert!(stdout.contains("EnvGlobal"));
    assert!(stdout.contains("saved token also present"));
    assert!(stdout.contains("InsecureFile"));
}

#[test]
fn auth_login_persists_credentials_across_invocations() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let mut login = bare_command(&temp, &host);
    login
        .env("GITQUARRY_ALLOW_INSECURE_STORAGE", "1")
        .args(["auth", "login", "--token-stdin"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = login.spawn().unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"fixture-token")
        .unwrap();
    let login_output = child.wait_with_output().unwrap();

    assert!(
        login_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&login_output.stderr)
    );

    let search = bare_command(&temp, &host)
        .env("GITQUARRY_ALLOW_INSECURE_STORAGE", "1")
        .args(["search", "rust cli", "--format", "json"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(
        search.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&search.stderr)
    );
}

#[test]
fn auth_logout_removes_insecure_credentials_without_opt_in_env() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();
    let credentials = format!("[hosts]\n\"{}\" = \"saved-token\"\n", host_key(&host));
    fs::write(temp.path().join("credentials.toml"), credentials).unwrap();

    let output = bare_command(&temp, &host)
        .args(["auth", "logout"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("logged out"));
    let credentials = fs::read_to_string(temp.path().join("credentials.toml")).unwrap();
    assert!(!credentials.contains("saved-token"));
}

#[test]
fn native_search_uses_only_one_search_request() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, request_count) = start_fixture_server();

    let output = base_command(&temp, &host)
        .args(["search", "rust cli", "--format", "json"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(request_count.load(Ordering::SeqCst), 1);
}

#[test]
fn deep_discover_skips_star_buckets_for_spaced_raw_stars_qualifier() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, request_count) = start_fixture_server();

    let output = base_command(&temp, &host)
        .args([
            "search",
            "rust Stars : >10",
            "--mode",
            "discover",
            "--depth",
            "deep",
            "--rank",
            "native",
            "--limit",
            "1",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(request_count.load(Ordering::SeqCst), 8);
}

#[test]
fn rank_requires_discover_mode() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = base_command(&temp, &host)
        .args(["search", "rust cli", "--rank", "blended"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("E_FLAG_REQUIRES_MODE"));
}

#[test]
fn weight_must_be_in_range() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = base_command(&temp, &host)
        .args([
            "search",
            "rust cli",
            "--mode",
            "discover",
            "--rank",
            "blended",
            "--weight-query",
            "4.2",
        ])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("E_FLAG_CONFLICT"));
}

#[test]
fn invalid_search_flags_fail_before_auth_resolution() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = bare_command(&temp, &host)
        .args(["search", "rust cli", "--rank", "blended"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("E_FLAG_REQUIRES_MODE"));
}

#[test]
fn invalid_inspect_target_fails_before_auth_resolution() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = bare_command(&temp, &host)
        .args(["inspect", "bad/repo/shape"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("repository must be in owner/repo form"));
}

#[test]
fn clap_parse_errors_are_wrapped_in_symbolic_error_codes() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = bare_command(&temp, &host)
        .args(["search", "rust cli", "--mode", "bogus"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("E_FLAG_PARSE"));
}

#[test]
fn config_path_uses_the_agent_config_dir() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = bare_command(&temp, &host)
        .args(["config", "path"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim(),
        temp.path().join("config.toml").display().to_string()
    );
}

#[test]
fn config_show_returns_json_payload() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = bare_command(&temp, &host)
        .args(["config", "show"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(output.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        payload["config_path"],
        temp.path().join("config.toml").display().to_string()
    );
}

#[test]
fn version_subcommand_prints_the_package_version() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = bare_command(&temp, &host)
        .args(["version"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim(),
        format!("gitquarry {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn completion_generation_matches_the_documented_shells() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = bare_command(&temp, &host)
        .args(["--generate-completion", "bash"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("_gitquarry()"));
}

#[test]
fn compact_output_is_minified_json() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = base_command(&temp, &host)
        .args(["search", "rust cli", "--format", "compact"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("\n  "));
    let payload: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(payload["items"][0]["full_name"], "example/rocket");
}

#[test]
fn csv_output_contains_header_and_repository_row() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = base_command(&temp, &host)
        .args(["inspect", "example/rocket", "--format", "csv"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("full_name,html_url,description,stars"));
    assert!(stdout.contains("example/rocket"));
}

#[test]
fn inspect_uses_last_contributor_page_not_per_page_parameter() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = base_command(&temp, &host)
        .args(["inspect", "example/big", "--format", "json"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(payload["repository"]["contributor_count"], 37);
}

#[test]
fn inspect_tolerates_large_repository_contributor_api_failures() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = base_command(&temp, &host)
        .args(["inspect", "example/huge", "--format", "json"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(payload["repository"]["contributor_count"].is_null());
}

#[test]
fn inspect_handles_empty_repository_contributor_response() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = base_command(&temp, &host)
        .args(["inspect", "example/empty", "--format", "json"])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(payload["repository"]["contributor_count"], 0);
}

#[test]
fn discover_search_handles_empty_repository_contributor_response() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = base_command(&temp, &host)
        .args([
            "search",
            "empty repo",
            "--mode",
            "discover",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(payload["items"][0]["full_name"], "example/empty");
    assert_eq!(payload["items"][0]["contributor_count"], 0);
}

#[test]
fn progress_on_writes_to_stderr_only() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = base_command(&temp, &host)
        .args([
            "search",
            "rust cli",
            "--mode",
            "discover",
            "--format",
            "json",
            "--progress",
            "on",
        ])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    serde_json::from_str::<serde_json::Value>(stdout.trim()).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("[gitquarry]"));
}

#[test]
fn progress_auto_stays_quiet_off_tty() {
    let temp = TempDir::new().unwrap();
    let (host, stop_tx, _) = start_fixture_server();

    let output = base_command(&temp, &host)
        .args([
            "search",
            "rust cli",
            "--mode",
            "discover",
            "--format",
            "json",
            "--progress",
            "auto",
        ])
        .output()
        .unwrap();

    stop_tx.send(()).ok();

    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stderr.contains("[gitquarry]"));
}
