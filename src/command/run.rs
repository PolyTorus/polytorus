use crate::command::term;
use clap::{App, SubCommand};
use crate::Result;

pub fn handle_term() -> Result<()> {
    let matches = App::new("run_tui")
        .subcommand(SubCommand::with_name("run_tui")) // サブコマンドの定義
        .get_matches();

    if let Some(ref _matches) = matches.subcommand_matches("run_tui") {
        term::run_tui()?;
    } else {
        term::run_tui()?;
    }
    Ok(())
}