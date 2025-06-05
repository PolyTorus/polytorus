use crate::command::cli_send::cmd_send_from_api;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct SendRequest {
    from: String,
    to: String,
    amount: i32,
    mine_now: bool,
    target_node: Option<String>,
}

#[post("/send")]
pub async fn send(req: web::Json<SendRequest>) -> impl Responder {
    let req_data = req.into_inner();
    let from: String = req_data.from.clone();
    let to: String = req_data.to.clone();
    let amount: i32 = req_data.amount.clone();
    let mine_now: bool = req_data.mine_now.clone();
    let target_node: Option<String> = req_data.target_node;

    tokio::task::spawn_blocking(move || {
        if let Err(e) = cmd_send_from_api(&from, &to, amount, mine_now, target_node.as_deref()) {
            eprintln!("Send start: {}", e);
        } else {
            println!("Send success");
        }
    });

    HttpResponse::Accepted().body("Send success")
}
