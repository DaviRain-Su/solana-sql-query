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
async fn health_check_works() -> anyhow::Result<()> {
    // Arrange
    let addr = spawn_app()?;
    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();
    // Act
    let response = client.get(format!("{}/health_check", addr)).send().await?;
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
    Ok(())
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() -> anyhow::Result<()> {
    // Arrange
    let app_address = spawn_app()?;
    let client = reqwest::Client::new();
    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;
    // Assert
    assert_eq!(200, response.status().as_u16());
    Ok(())
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() -> anyhow::Result<()> {
    // Arrange
    let app_address = spawn_app()?;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await?;
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
    Ok(())
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
