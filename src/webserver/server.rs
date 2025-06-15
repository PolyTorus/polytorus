use std::sync::Arc;

use actix_web::{
    web,
    App,
    HttpServer,
};
use tokio::sync::mpsc;

use crate::network::NetworkCommand;
use crate::webserver::createwallet;
use crate::webserver::listaddresses;
use crate::webserver::network_api::{
    blacklist_peer,
    get_message_queue_stats,
    get_network_health,
    get_peer_info,
    unblacklist_peer,
    NetworkApiState,
};
use crate::webserver::printchain;
use crate::webserver::reindex;
use crate::webserver::startminer;
use crate::webserver::startnode;

pub struct WebServer {}

impl WebServer {
    pub async fn run() -> std::io::Result<()> {
        // Create a dummy network command channel for demonstration
        // In a real application, this would be connected to the actual network node
        let (tx, _rx) = mpsc::unbounded_channel::<NetworkCommand>();
        let network_api_state = Arc::new(NetworkApiState::new(tx));

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(network_api_state.clone()))
                .service(createwallet::create_wallet)
                .service(printchain::print_chain)
                .service(listaddresses::list_addresses)
                .service(reindex::reindex)
                .service(startnode::start_node)
                .service(startminer::start_miner)
                // Network API endpoints
                .service(get_network_health)
                .service(get_peer_info)
                .service(get_message_queue_stats)
                .service(blacklist_peer)
                .service(unblacklist_peer)
        })
        .bind(("127.0.0.1", 7000))?
        .run()
        .await
    }
}
