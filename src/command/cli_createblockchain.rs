use crate::blockchain::{blockchain::Blockchain, utxoset::UTXOSet};
use crate::Result;

pub fn cmd_create_blockchain_from_api(address: &str) -> Result<()> {
    let address = String::from(address);
    let bc = Blockchain::create_blockchain(address)?;

    let utxo_set = UTXOSet { blockchain: bc };
    utxo_set.reindex()?;
    println!("create blockchain");
    Ok(())
}
