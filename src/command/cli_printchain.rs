use crate::blockchain::blockchain::Blockchain;
use failure::Error;

pub fn cmd_print_chain() -> Result<(), Error> {
    let bc = Blockchain::new()?;
    for b in bc.iter() {
        println!("{:#?}", b);
    }
    Ok(())
}