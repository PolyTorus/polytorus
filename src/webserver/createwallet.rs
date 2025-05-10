use crate::command::cli_createwallet::cmd_create_wallet;
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

#[post("/create-wallet/{encryption}")]
pub async fn create_wallet_with_param(path: web::Path<CryptoPath>) -> impl Responder {
    let enc_str = &path.encryption;
    handle_create_wallet(enc_str)
}

#[post("/create-wallet")]
pub async fn create_wallet_default() -> impl Responder {
    let default = "FNDSA";
    handle_create_wallet(default)
}

fn handle_create_wallet(enc_str: &str) -> HttpResponse {
    match enc_str.parse::<EncryptionType>() {
        Ok(encryption) => match cmd_create_wallet(encryption) {
            Ok(msg) => HttpResponse::Ok().body(msg),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        },
        Err(_) => HttpResponse::BadRequest().body("不正な暗号方式です"),
    }
}
