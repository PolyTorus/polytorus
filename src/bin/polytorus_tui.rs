//! Polytorus TUI application binary

use polytorus::tui::TuiApp;

#[tokio::main]
async fn main() -> polytorus::Result<()> {
    // Initialize logging
    env_logger::init();

    // Run the TUI application
    TuiApp::run().await
}
