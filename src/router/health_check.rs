use actix_web::HttpResponse;

pub async fn health_check() -> HttpResponse {
    tracing::info!("health_check");
    HttpResponse::Ok().finish()
}
