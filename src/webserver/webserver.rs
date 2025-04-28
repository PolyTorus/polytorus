use crate::webserver::createwallet;
use crate::webserver::printchain;
use actix_web::{App, HttpServer};

use super::remotesend;

pub struct WebServer {}

impl WebServer {
    pub async fn new() -> std::io::Result<()> {
        HttpServer::new(|| {
            App::new()
                .service(createwallet::create_wallet)
                .service(printchain::print_chain)
                .service(remotesend::remote_send)
        })
        .bind(("127.0.0.1", 7000))?
        .run()
        .await
    }
}
