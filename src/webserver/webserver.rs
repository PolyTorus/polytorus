use crate::command::cli::cmd_create_wallet;
use crate::crypto::types::EncryptionType;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::str::FromStr;

pub struct WebServer {}

impl WebServer {
    pub async fn new() -> std::io::Result<()> {
        HttpServer::new(|| App::new().service(hello))
            .bind(("127.0.0.1", 7000))?
            .run()
            .await
    }
}

impl FromStr for EncryptionType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ECDSA" => Ok(EncryptionType::ECDSA),
            "FNDSA" => Ok(EncryptionType::FNDSA),
            _ => Err(()),
        }
    }
}

#[derive(Deserialize)]
struct CryptoPath {
    encryption: String,
}

#[post("/create_wallet/{encryption}")]
async fn hello(path: web::Path<CryptoPath>) -> impl Responder {
    match path.encryption.parse::<EncryptionType>() {
        Ok(encryption) => match cmd_create_wallet(encryption) {
            Ok(msg) => HttpResponse::Ok().body(msg),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        },
        Err(_) => HttpResponse::BadRequest().body("不正な暗号方式です"),
    }
}
