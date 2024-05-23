use actix_web::dev::Server;
use actix_web::HttpResponse;
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;

pub async fn health_check() -> HttpResponse {
    tracing::info!("health_check");
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

// Let's start simple: we always return a 200 OK
async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> anyhow::Result<Server> {
    tracing::info!(
        "Starting server listening on http://{}",
        listener.local_addr()?
    );
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            // A new entry in our routing table for POST /subscriptions requests
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
