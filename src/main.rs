use actix_web::{App, HttpServer, web};
use polytorus::app::p2p::P2p;
use polytorus::app::route::index;
use polytorus::app::show_block::block;
use polytorus::app::mine::mine;
use polytorus::app::transaction::transactions;
use polytorus::app::transact::transact;
use polytorus::app::public_key::public_key;
use polytorus::app::global::{CHAIN, start_p2p, SERVER};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::clone::Clone;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let blockchain: Arc<CHAIN> = Arc::new(CHAIN.clone());
    let server: Arc<Mutex<P2p>> = Arc::new(Mutex::new(SERVER.clone()));

    let p2p_port: String = std::env::var("P2P_PORT").unwrap_or_else(|_| "5001".to_string());
    let http_port: String = std::env::var("HTTP_PORT").unwrap_or_else(|_| "3001".to_string());

    start_p2p().await;

    let server_clone_for_spawn = server.clone();
    tokio::spawn(async move {
        if let Err(e) = server_clone_for_spawn.lock().await.connect_peers().await {
            eprintln!("Error connecting to peers: {}", e);
        }
    });

    println!("Start http server: http://localhost:{}", http_port);
    println!("Start p2p server: ws://localhost:{}", p2p_port);


    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(blockchain.clone()))
            .app_data(web::Data::new(server.clone()))
            .service(index)
            .service(block)
            .service(mine)
            .service(transactions)
            .service(transact)
            .service(public_key)
            .service(web::redirect("/mine", "/block"))
            .service(web::redirect("/trasact", "/transactions"))
    })
    .bind(format!("0.0.0.0:{}", http_port))?
    .run()
    .await
}
