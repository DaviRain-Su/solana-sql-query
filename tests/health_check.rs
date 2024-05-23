//! tests/health_check.rs
use std::net::TcpListener;

use solana_query_service::health_check;

#[tokio::test]
async fn test_health_check() {
    let response = health_check().await;
    assert_eq!(response.status(), 200);
}

/// `tokio::test` is the testing equivalent of `tokio::main`.
// It also spares you from having to specify the `#[test]` attribute.
//
// You can inspect what code gets generated using
// `cargo expand --test health_check` (<- name of the test file)
#[tokio::test]
async fn health_check_works() {
    // Arrange
    let addr = spawn_app().expect("Failed to spawn our app.");
    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();
    // Act
    let response = client
        .get(format!("{}/health_check", addr))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// Launch our application in the background ~somehow~
fn spawn_app() -> anyhow::Result<String> {
    //
    let listener = TcpListener::bind("127.0.0.1:0")?;
    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr()?.port();
    let server = solana_query_service::run(listener)?;
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);
    // We return the application address to the caller!
    Ok(format!("http://127.0.0.1:{}", port))
}
