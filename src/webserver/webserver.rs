use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

pub struct WebServer {}

impl WebServer {
    pub async fn new() -> std::io::Result<()> {
        HttpServer::new(|| App::new().service(hello))
            .bind(("127.0.0.1", 7000))?
            .run()
            .await
    }
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
