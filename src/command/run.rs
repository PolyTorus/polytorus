use crate::command::term;

pub fn handle_term(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
  if args.contains(&String::from("--term")) {
    term::run_tui()?; // Changed to run_tui to correctly call the TUI entry point
  } else {
    println!("CLI mode");
  }
  Ok(())
}
