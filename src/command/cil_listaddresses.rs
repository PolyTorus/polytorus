use crate::crypto::wallets::Wallets;

pub fn cmd_list_address() -> Result<(),Box<dyn std::error::Error>> {
    let ws = Wallets::new()?;
    let addresses = ws.get_all_addresses();
    println!("addresses: ");
    for ad in addresses {
        println!("{}", ad);
    }
    Ok(())
}
