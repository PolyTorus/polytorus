// Legacy command removed - reindex functionality not available in modern CLI
use actix_web::{
    post,
    HttpResponse,
    Responder,
};

#[post("/reindex")]
pub async fn reindex() -> impl Responder {
    HttpResponse::NotImplemented()
        .body("Reindex functionality has been removed in modern architecture")
}
