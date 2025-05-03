use crate::blockchain::{blockchain::Blockchain, utxoset::UTXOSet};
use bitcoincash_addr::Address;

pub fn cli_get_balance(sub_m: Option<&str>) -> Result<i32, String> {
    if let Some(address) = sub_m {
        let balance = cmd_get_balance(address).map_err(|e| e.to_string())?;
        Ok(balance)
    } else {
        Err("Failed to get balance".to_string())
    }
}

fn cmd_get_balance(address: &str) -> crate::Result<i32> {
    let pub_key_hash = Address::decode(address).unwrap().body;
    let bc = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };
    let utxos = utxo_set.find_UTXO(&pub_key_hash)?;

    let balance = utxos.outputs.iter().map(|out| out.value).sum();
    println!("Balance: {}\n", balance);
    Ok(balance)
}
