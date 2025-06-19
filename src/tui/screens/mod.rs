//! Screen modules for the TUI

pub mod dashboard;
pub mod network;
pub mod transactions;
pub mod wallets;

pub use dashboard::DashboardScreen;
pub use network::NetworkScreen;
pub use transactions::TransactionsScreen;
pub use wallets::WalletsScreen;
