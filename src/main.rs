use actix_web::{App, HttpServer, web};
use polytorus::app::p2p::P2p;
use polytorus::app::route::index;
use polytorus::app::show_block::block;
use polytorus::blockchain::chain::Chain;
use polytorus::app::mine::mine;
use std::sync::Arc;
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let blockchain = Chain::new();
    let server = Arc::new(Mutex::new(P2p::new(blockchain.clone())));

    let p2p_port = std::env::var("P2P_PORT").unwrap_or_else(|_| "8081".to_string());
    let server_clone = server.clone();
    let _ = server.lock().await.connect_peers().await;

    let server_clone_for_spawn = server_clone.clone();
    tokio::spawn(async move {
        if let Err(e) = server_clone_for_spawn.lock().await.connect_peers().await {
            eprintln!("Error connecting to peers: {}", e);
        }
    });

    HttpServer::new(move || {
        let server_clone = server_clone.clone();
        App::new()
            .app_data(web::Data::new(blockchain.clone()))
            .app_data(web::Data::new(server_clone))
            .service(index)
            .service(block)
            .service(mine)
            .service(web::redirect("/mine", "/block"))
    })
    .bind(format!("127.0.0.1:{}", p2p_port))?
    .run()
    .await
}
