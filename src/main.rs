use polytorus::command::cli::Cli;
use env_logger::Env;

fn main() {
    env_logger::from_env(Env::default().default_filter_or("warning")).init();

    let mut cli = Cli::new();
    if let Err(e) = cli.run() {
        println!("Error: {}", e);
    }
}
