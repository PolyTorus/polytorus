//! Transactions screen

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::tui::{
    components::{StatusBarComponent, TransactionListComponent},
    utils::{NetworkStats, TransactionInfo, TransactionStatus},
    vim_mode::VimMode,
};

#[derive(Clone)]
pub struct TransactionsScreen {
    pub transaction_list: TransactionListComponent,
    pub network_stats: NetworkStats,
}

impl Default for TransactionsScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionsScreen {
    pub fn new() -> Self {
        Self {
            transaction_list: TransactionListComponent::new(),
            network_stats: NetworkStats::default(),
        }
    }

    pub fn with_transactions(mut self, transactions: Vec<TransactionInfo>) -> Self {
        self.transaction_list = self.transaction_list.with_transactions(transactions);
        self
    }

    pub fn add_transaction(&mut self, transaction: TransactionInfo) {
        self.transaction_list.add_transaction(transaction);
    }

    pub fn update_transaction_status(&mut self, hash: &str, status: TransactionStatus) {
        self.transaction_list
            .update_transaction_status(hash, status);
    }

    pub fn update_network_stats(&mut self, stats: NetworkStats) {
        self.network_stats = stats;
    }

    pub fn selected_transaction(&self) -> Option<&TransactionInfo> {
        self.transaction_list.selected_transaction()
    }

    pub fn next_transaction(&mut self) {
        self.transaction_list.next();
    }

    pub fn previous_transaction(&mut self) {
        self.transaction_list.previous();
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool, vim_mode: &VimMode) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Main content
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        // Render transaction list
        self.transaction_list.render(frame, main_chunks[0], focused);

        // Status bar
        let mut status_bar = StatusBarComponent::new();
        status_bar.update_network_stats(self.network_stats.clone());
        status_bar.set_current_screen("Transactions".to_string());
        status_bar.set_vim_mode(vim_mode.clone());
        status_bar.render(frame, main_chunks[1]);
    }
}
