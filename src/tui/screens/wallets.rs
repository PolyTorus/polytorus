//! Wallets screen

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::tui::{
    components::{StatusBarComponent, WalletListComponent},
    utils::{NetworkStats, WalletInfo},
    vim_mode::VimMode,
};

#[derive(Clone)]
pub struct WalletsScreen {
    pub wallet_list: WalletListComponent,
    pub network_stats: NetworkStats,
}

impl Default for WalletsScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl WalletsScreen {
    pub fn new() -> Self {
        Self {
            wallet_list: WalletListComponent::new(),
            network_stats: NetworkStats::default(),
        }
    }

    pub fn with_wallets(mut self, wallets: Vec<WalletInfo>) -> Self {
        self.wallet_list = self.wallet_list.with_wallets(wallets);
        self
    }

    pub fn add_wallet(&mut self, wallet: WalletInfo) {
        self.wallet_list.add_wallet(wallet);
    }

    pub fn update_network_stats(&mut self, stats: NetworkStats) {
        self.network_stats = stats;
    }

    pub fn selected_wallet(&self) -> Option<&WalletInfo> {
        self.wallet_list.selected_wallet()
    }

    pub fn next_wallet(&mut self) {
        self.wallet_list.next();
    }

    pub fn previous_wallet(&mut self) {
        self.wallet_list.previous();
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool, vim_mode: &VimMode) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Main content
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        // Render wallet list
        self.wallet_list.render(frame, main_chunks[0], focused);

        // Status bar
        let mut status_bar = StatusBarComponent::new();
        status_bar.update_network_stats(self.network_stats.clone());
        status_bar.set_current_screen("Wallets".to_string());
        status_bar.set_vim_mode(vim_mode.clone());
        status_bar.render(frame, main_chunks[1]);
    }
}
