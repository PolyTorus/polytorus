use crate::blockchain::utxoset::UTXOSet;
use crate::blockchain::blockchain::Blockchain;
use crate::network::server::Server;
use failure::Error;

pub fn cmd_start_miner_from_api(
    host: &str,
    port: &str,
    bootstrap: Option<&str>,
    mining_address: &str,
) -> Result<(), Error> {
    println!("Start miner node...");

    let bc = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };
    let server = Server::new(host, port, mining_address, bootstrap, utxo_set)?;

    std::thread::spawn(move || {
        if let Err(e) = server.start_server() {
            eprintln!("Failed to run miner server: {}", e);
        } else {
            println!("Miner server started");
        }
    });

    Ok(())
}