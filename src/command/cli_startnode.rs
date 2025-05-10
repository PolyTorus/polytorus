use crate::blockchain::blockchain::Blockchain;
use crate::blockchain::utxoset::UTXOSet;
use crate::network::server::Server;
use failure::Error;

pub fn cmd_start_node_from_api(
    host: &str,
    port: &str,
    bootstrap: Option<&str>,
) -> Result<(), Error> {
    println!("Start node...");

    let bc = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };
    let server = Server::new(host, port, "", bootstrap, utxo_set)?;

    std::thread::spawn(move || {
        if let Err(e) = server.start_server() {
            eprintln!("Failed to run node: {}", e);
        }
    });

    Ok(())
}
