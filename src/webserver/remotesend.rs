use crate::command::cli::cmd_remote_send;
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
pub async fn remote_send(params: web::Query<RemoteSendParams>) -> impl Responder {
    let from = &params.from;
    let to = &params.to;
    let amount = params.amount;
    let node = &params.node;
    let mine = params.mine;
    match cmd_remote_send(from, to, amount, node, mine) {
        Ok(()) => HttpResponse::Ok().body("Complete remote send"),
        Err(err) => HttpResponse::BadGateway().body(err.to_string()),
    }
}
