use actix_web::{App, HttpServer};
use polytorus::app::route::index;
use polytorus::app::show_block::block;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(block)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}