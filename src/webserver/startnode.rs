use actix_web::{post, web, HttpResponse, Responder};
use crate::command::cil_startnode::cmd_start_node_from_api;
use serde::Deserialize;

#[derive(Deserialize)]
struct StartNodeRequest {
    host: String,
    port: String,
    bootstrap: Option<String>,
}

#[post("/start-node")]
pub async fn start_node(req: web::Json<StartNodeRequest>) -> impl Responder {
    let req_data = req.into_inner();
    println!("@start-Node called: host={}, port={}", req_data.host, req_data.port);

    let host = req_data.host.clone();
    let port = req_data.port.clone();
    let bootstrap = req_data.bootstrap.clone();

    tokio::task::spawn_blocking(move || {
        if let Err(e) = cmd_start_node_from_api(&host, &port, bootstrap.as_deref()) {
            eprintln!("Node failed to start: {}", e);
        } else {
            println!("Node started successfully");
        }
    });

    HttpResponse::Accepted().body("Node is starting in background")
}