use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

// 動作確認用
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
    })
    .bind(("127.0.0.1", 7000))?
    .run()
    .await
}
