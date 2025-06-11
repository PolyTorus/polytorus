use crate::command::cli::ModernCli;
use crate::crypto::types::EncryptionType;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::str::FromStr;

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
pub async fn create_wallet(path: web::Path<CryptoPath>) -> impl Responder {
    match path.encryption.parse::<EncryptionType>() {
        Ok(_) => {
            let cli = ModernCli::new();
            match cli.cmd_create_wallet().await {
                Ok(_) => HttpResponse::Ok().body("Wallet created successfully"),
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            }
        },
        Err(_) => HttpResponse::BadRequest().body("Invalid encryption type"),
    }
}
