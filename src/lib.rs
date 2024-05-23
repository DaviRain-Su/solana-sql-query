use actix_web::dev::Server;
use actix_web::HttpResponse;
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;

pub async fn health_check() -> HttpResponse {
    tracing::info!("health_check");
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> anyhow::Result<Server> {
    tracing::info!(
        "Starting server listening on http://{}",
        listener.local_addr()?
    );
    let server = HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .listen(listener)?
        .run();

    Ok(server)
}
