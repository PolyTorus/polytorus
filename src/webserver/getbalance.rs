use crate::{
    command::{cli::cmd_create_wallet, cli_getbalance::cli_get_balance},
    crypto::types::EncryptionType,
};
use actix_web::{http, post, test, web, App, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct AddressQuery {
    address: String,
}

#[post("/get_balance")]
pub async fn get_balance(sub_m: web::Query<AddressQuery>) -> impl Responder {
    get_balance_handler(sub_m).await
}

pub async fn get_balance_handler(sub_m: web::Query<AddressQuery>) -> impl Responder {
    match cli_get_balance(Some(&sub_m.address)) {
        Ok(balance) => HttpResponse::Ok().body(format!("Complete get balance: {}", balance)),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[actix_web::test]
async fn test_get_balance() {
    let _ = crate::webserver::webserver::WebServer::new();
    let addr1 = cmd_create_wallet(EncryptionType::FNDSA).unwrap_or("ERROR!".to_string());
    let addr1 = web::Query(AddressQuery { address: addr1 });
    let app = test::init_service(App::new().route(
        "/get_balance",
        web::post().to(move || get_balance_handler(addr1.clone())),
    ))
    .await;
    let req = test::TestRequest::post().uri("/get_balance").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), http::StatusCode::OK);

    let body = test::read_body(resp).await;
    let body_str =
        String::from_utf8(body.to_vec()).expect("Failed to convert response body to UTF-8");
    let balance = 0;
    assert_eq!(body_str, format!("Complete get balance: {}", balance));
}
