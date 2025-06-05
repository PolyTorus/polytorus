use crate::blockchain::{blockchain::Blockchain, utxoset::UTXOSet};
use crate::crypto::{fndsa::FnDsaCrypto, transaction::Transaction, wallets::Wallets};
use crate::network::server::Server;
use crate::Result;

pub fn cmd_send_from_api(
    from: &str,
    to: &str,
    amount: i32,
    mine_now: bool,
    target_node: Option<&str>,
) -> Result<()> {
    let bc = Blockchain::new()?;
    let mut utxo_set = UTXOSet { blockchain: bc };
    let wallets = Wallets::new()?;
    let wallet = wallets.get_wallet(from).unwrap();
    // TODO: 暗号化方式を選択
    let crypto = FnDsaCrypto;
    let tx = Transaction::new_UTXO(wallet, to, amount, &utxo_set, &crypto)?;
    if mine_now {
        let cbtx = Transaction::new_coinbase(from.to_string(), String::from("reward!"))?;
        let new_block = utxo_set.blockchain.mine_block(vec![cbtx, tx])?;

        utxo_set.update(&new_block)?;
    } else {
        Server::send_transaction(&tx, utxo_set, target_node.unwrap_or("0.0.0.0:7000"))?;
    }

    println!("success!");
    Ok(())
}
