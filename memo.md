```rs
// cil_startminer
use crate::blockchain::utxoset::UTXOSet;
use crate::blockchain::blockchain::Blockchain;
use crate::network::server::Server;
use failure::Error;

pub fn cmd_start_miner_from_api(host: &str,port:&str, bootstrap: Option<&str>,mining_address:&str) -> Result<(), Error> {
    println!("Start miner node...");
    let bc = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };
    let server = Server::new(host, port,mining_address,bootstrap,utxo_set,)?;
    server.start_server()?;
    Ok(())
}


// startminer.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::command::cil_startminer::cmd_start_miner_from_api;
use serde::Deserialize;
use std::thread;

#[derive(Deserialize)]
struct StartMinerRequest {
    host: String,
    port: String,
    bootstrap: Option<String>,
    mining_address: String
}

#[post("/start-miner")]
pub async fn start_miner(req: web::Json<StartMinerRequest>) -> impl Responder {
    let req_data = req.into_inner();
    println!("@start-Mner called: host={}, port={}", req_data.host, req_data.port);

    thread::spawn(move || {
        if let Err(e) = cmd_start_miner_from_api(&req_data.host, &req_data.port, req_data.bootstrap.as_deref(), &req_data.mining_address) {
            println!("Failed to start miner: {}", e);
        }
    });

    HttpResponse::Ok().body("Miner starting...")
}

// startnode.rs
use actix_web::{post, web, HttpResponse, Responder};
use crate::command::cil_startnode::cmd_start_node_from_api;
use serde::Deserialize;
use std::thread;

#[derive(Deserialize)]
struct StartNodeRequest {
    host: String,
    port: String,
    bootstrap: Option<String>, // 必要なら
}

#[post("/start-node")]
pub async fn start_node(req: web::Json<StartNodeRequest>) -> impl Responder {
    let req_data = req.into_inner();
    println!("@start-node called: host={}, port={}", req_data.host, req_data.port);

    thread::spawn(move || {
        if let Err(e) = cmd_start_node_from_api(&req_data.host, &req_data.port, req_data.bootstrap.as_deref()) {
            println!("Failed to start node: {}", e);
        }
    });

    HttpResponse::Ok().body("Node starting...")
}


//cil_startnode.rs
use crate::blockchain::utxoset::UTXOSet;
use crate::blockchain::blockchain::Blockchain;
use crate::network::server::Server;
use failure::Error;

pub fn cmd_start_node_from_api(host: &str, port: &str, bootstrap: Option<&str>) -> Result<(), Error> {
    println!("Start node...");

    let bc = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };
    let server = Server::new(host, port, "", bootstrap, utxo_set)?;

    // server を move して新しいスレッドで start_server を呼び出す
    std::thread::spawn(move || {
        if let Err(e) = server.start_server() {
            eprintln!("Failed to run node: {}", e);
        }
    });

    // ここで即座に成功を返せるようになる
    Ok(())
}
```