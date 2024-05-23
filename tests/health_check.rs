//! tests/health_check.rs
use once_cell::sync::Lazy;
use sqlx::Connection;
use sqlx::Executor;
use sqlx::PgConnection;
use sqlx::PgPool;
use std::net::TcpListener;
use uuid::Uuid;

use solana_query_service::configuration::get_configuration;
use solana_query_service::configuration::DatabaseSettings;
use solana_query_service::router::health_check;
use solana_query_service::telemetry::{get_subscriber, init_subscriber};

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // We cannot assign the output of `get_subscriber` to a variable based on the
    // value TEST_LOG` because the sink is part of the type returned by
    // `get_subscriber`, therefore they are not the same type. We could work around
    // it, but this is the most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(default_filter_level, subscriber_name, std::io::stdout);
        init_subscriber(subscriber).unwrap();
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber).unwrap();
    }
});

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
    let test_app = spawn_app().await?;
    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();
    // Act
    let response = client
        .get(format!("{}/health_check", test_app.address))
        .send()
        .await?;
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
    Ok(())
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() -> anyhow::Result<()> {
    // Arrange
    let test_app = spawn_app().await?;
    let client = reqwest::Client::new();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await?;

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

    Ok(())
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() -> anyhow::Result<()> {
    // Arrange
    let test_app = spawn_app().await?;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
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

#[derive(Debug)]
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// Launch our application in the background ~somehow~
async fn spawn_app() -> anyhow::Result<TestApp> {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0")?;
    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr()?.port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut configuration = get_configuration()?;
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await?;
    let server = solana_query_service::startup::run(listener, connection_pool.clone())?;

    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);
    // We return the application address to the caller!
    Ok(TestApp {
        address,
        db_pool: connection_pool,
    })
}

pub async fn configure_database(config: &DatabaseSettings) -> anyhow::Result<PgPool> {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_db()).await?;
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await?;
    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string()).await?;
    sqlx::migrate!("./migrations").run(&connection_pool).await?;

    Ok(connection_pool)
}
