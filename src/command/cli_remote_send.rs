use crate::blockchain::{blockchain::Blockchain, utxoset::UTXOSet};
use crate::crypto::transaction::{TXOutput, Transaction};
use crate::network::server::Server;
use crate::Result;

pub fn cmd_remote_send(
    from: &str,
    to: &str,
    amount: i32,
    node: &str,
    _mine_now: bool,
) -> Result<()> {
    let bc = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };

    let tx = Transaction {
        id: String::new(),
        vin: Vec::new(),
        vout: vec![TXOutput::new(amount, to.to_string())?],
    };

    let server = Server::new("0.0.0.0", "0", "", None, utxo_set)?;

    let signed_tx = server.send_sign_request(node, from, &tx)?;

    server.send_tx(node, &signed_tx)?;

    println!("Transaction sent successfully!");
    Ok(())
}
