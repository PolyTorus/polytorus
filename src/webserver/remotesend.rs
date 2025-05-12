use crate::command::cli_remote_send::cmd_remote_send;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct RemoteSendParams {
    from: String,
    to: String,
    amount: i32,
    node: String,
    mine: bool,
}

#[post("/remote_send")]
pub async fn remote_send(req: web::Json<RemoteSendParams>) -> impl Responder {
    let req_data = req.into_inner();
    let from = req_data.from.clone();
    let to = req_data.to.clone();
    let amount = req_data.amount.clone();
    let node = req_data.node.clone();
    let mine = req_data.mine.clone();
    tokio::task::spawn_blocking(move || {
        if let Err(e) = cmd_remote_send(&from, &to, amount, &node, mine) {
            eprintln!("Miner failed: {}", e);
        } else {
            println!("Remote_send success");
        }
    });

    HttpResponse::Accepted().body("Remote_send success")
}
