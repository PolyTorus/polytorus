use env_logger::Env;
use polytorus::command::cli::Cli;
use actix_web;

#[actix_web::main]
async fn main() {
    env_logger::from_env(Env::default().default_filter_or("warning")).init();

    let mut cli = Cli::new();
    if let Err(e) = cli.run().await {
        println!("Error: {}", e);
    }
}
