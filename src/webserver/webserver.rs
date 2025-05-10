use crate::webserver::createwallet;
use crate::webserver::listaddresses;
use crate::webserver::printchain;
use crate::webserver::reindex;
use crate::webserver::startminer;
use crate::webserver::startnode;
use actix_web::{App, HttpServer};

pub struct WebServer {}

impl WebServer {
    pub async fn new() -> std::io::Result<()> {
        HttpServer::new(|| {
            App::new()
                .service(createwallet::create_wallet_with_param)
                .service(createwallet::create_wallet_default)
                .service(printchain::print_chain)
                .service(listaddresses::list_addresses)
                .service(reindex::reindex)
                .service(startnode::start_node)
                .service(startminer::start_miner)
        })
        .bind(("127.0.0.1", 7000))?
        .run()
        .await
    }
}
