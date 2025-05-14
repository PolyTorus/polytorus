use crate::crypto::wallets::Wallets;
use crate::crypto::types::EncryptionType;
use failure::Error;

pub fn cmd_create_wallet(encryption: EncryptionType) -> Result<String,Error> {
    let mut ws = Wallets::new()?;
    let address = ws.create_wallet(encryption);
    ws.save_all()?;
    println!("address: {}",address);
    
    Ok(address)
}