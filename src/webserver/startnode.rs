// Legacy CLI command removed - use modular architecture
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
struct StartNodeRequest {
    host: String,
    port: String,
    bootstrap: Option<String>,
}

#[post("/start-node")]
pub async fn start_node(_req: web::Json<StartNodeRequest>) -> impl Responder {
    HttpResponse::NotImplemented()
        .body("Legacy node has been removed. Use 'polytorus modular start' commands instead.")
}
