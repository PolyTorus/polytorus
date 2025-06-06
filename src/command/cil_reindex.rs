use crate::blockchain::blockchain::Blockchain;
use crate::blockchain::types::network;
use crate::blockchain::utxoset::UTXOSet;
use failure::Error;

pub fn cmd_reindex() -> Result<(), Error> {
    let bc: Blockchain<network::Mainnet> = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };
    utxo_set.reindex()?;
    utxo_set.count_transactions()?;
    Ok(())
}
