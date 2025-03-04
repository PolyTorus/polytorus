use env_logger::Env;
use polytorus::command::cli::Cli;
use polytorus::command::run; //add run module

fn main() -> Result<(), Box<dyn std::error::Error>> { //add Result type
    env_logger::from_env(Env::default().default_filter_or("warning")).init();

    let mut cli = Cli::new();
    if let Err(e) = cli.run() {
        println!("Error: {}", e);
    }

    let args: Vec<String> = std::env::args().collect();
    run::handle_term(args)?; //call run::handle_term
    Ok(())
}
