use env_logger::Env;
use polytorus::command::cli::Cli;
use polytorus::command::term;

fn main() {
    env_logger::from_env(Env::default().default_filter_or("warning")).init();

    let mut cli = Cli::new();
    if let Err(e) = cli.run() {
        println!("Error: {}", e);
    }

    let args: Vec<String> = std::env::args().collect();
    if args.contains(&String::from("--term")) {
        if let Err(e) = term::run_tui() {
            println!("Error: {}", e);
        }
    } else {
        println!("CLI mode");
    }
}
