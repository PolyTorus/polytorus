use crate::command::term;
use clap::App;
use crate::Result;


pub fn handle_term() -> Result<()> {
    let matches = App::new("run_tui")
        .get_matches();
    if let Some(ref _matches) = matches.subcommand_matches("run_tui") {
        term::run_tui()?;
    } else {
        term::run_tui()?;
    }
    Ok(())
}
