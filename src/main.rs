use actix_web::{App, HttpServer, web};
use polytorus::app::p2p::P2p;
use polytorus::app::route::index;
use polytorus::app::show_block::block;
use polytorus::blockchain::chain::Chain;
use polytorus::app::mine::mine;
use polytorus::app::transaction::transactions;
use polytorus::app::transact::transact;
use polytorus::app::global::start_p2p;
use polytorus::app::public_key::public_key;
use std::sync::Arc;
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let blockchain: Chain = Chain::new();
    let server: Arc<Mutex<P2p>> = Arc::new(Mutex::new(P2p::new(blockchain.clone())));

    let p2p_port: String = std::env::var("P2P_PORT").unwrap_or_else(|_| "5001".to_string());
    let http_port: String = std::env::var("HTTP_PORT").unwrap_or_else(|_| "3001".to_string());
    // let server_clone = server.clone();
    // let _ = server.lock().await.connect_peers().await;

    // let p2p_server = server.clone();
    // tokio::spawn(async move {
    //     if let Err(e) = p2p_server.lock().await.listen().await {
    //         eprintln!("Error listening: {}", e);
    //     }
    // });

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
        let server_clone = server.clone();
        App::new()
            .app_data(web::Data::new(blockchain.clone()))
            .app_data(web::Data::new(server_clone))
            .service(index)
            .service(block)
            .service(mine)
            .service(transactions)
            .service(transact)
            .service(public_key)
            .service(web::redirect("/mine", "/block"))
            .service(web::redirect("/trasact", "/transactions"))
    })
    .bind(format!("127.0.0.1:{}", http_port))?
    .run()
    .await
}
