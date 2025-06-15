// Modern CLI integration
use actix_web::{
    post,
    HttpResponse,
    Responder,
};

use crate::command::cli::ModernCli;

#[post("/list-addresses")]
pub async fn list_addresses() -> impl Responder {
    let cli = ModernCli::new();
    match cli.cmd_list_addresses().await {
        Ok(()) => HttpResponse::Ok().body("Complete list addresses"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
