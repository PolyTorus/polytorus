pub fn handle_term(args: Vec<String>) {
  if args.contains(&String::from("--term")) {
    term::tui_print_chain();
  } else {
    println!("CLI mode");
  }
}