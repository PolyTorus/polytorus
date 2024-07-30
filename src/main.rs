use actix_web::{App, HttpServer, web};
use polytorus::app::route::index;
use polytorus::app::show_block::block;
use polytorus::app::mine::mine;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(block)
            .service(mine)
            .service(web::redirect("/mine", "/block"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}