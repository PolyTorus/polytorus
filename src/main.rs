use env_logger::Env;
use polytorus::command::cli::ModernCli;

/// PolyTorus - Post Quantum Modular Blockchain
///
/// This is the main entry point for the PolyTorus blockchain platform.
/// The platform is built on a modular architecture with separate layers
/// for execution, settlement, consensus, and data availability.
#[actix_web::main]
async fn main() {
    // Initialize logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    println!("🔗 PolyTorus - Post Quantum Modular Blockchain");
    println!("📝 For help: polytorus --help");
    println!("🚀 Quick start: polytorus modular start");
    println!();

    let cli = ModernCli::new();
    if let Err(e) = cli.run().await {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}
