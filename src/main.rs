use actix_web::{App, HttpServer, web};
use polytorus::app::p2p::websocket_route;
use polytorus::app::route::index;
use polytorus::app::show_block::block;
use polytorus::blockchain::chain::Chain;
use polytorus::app::mine::mine;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let blockchain = Chain::new();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(blockchain.clone()))
            .service(index)
            .service(block)
            .service(mine)
            .service(web::redirect("/mine", "/block"))
            .route("/ws/", web::get().to(websocket_route))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}