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
    let from = &req_data.from;
    let to = &req_data.to;
    let amount = req_data.amount;
    let node = &req_data.node;
    let mine = req_data.mine;
    match cmd_remote_send(from, to, amount, node, mine) {
        Ok(()) => HttpResponse::Ok().body("Complete remote send"),
        Err(err) => HttpResponse::BadGateway().body(err.to_string()),
    }
}
