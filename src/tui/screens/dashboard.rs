//! Dashboard screen

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::{
    components::StatusBarComponent,
    styles::AppStyles,
    utils::{format_balance, NetworkStats},
    vim_mode::VimMode,
};

pub struct DashboardScreen {
    pub total_balance: u64,
    pub wallet_count: usize,
    pub transaction_count: usize,
    pub network_stats: NetworkStats,
}

impl Default for DashboardScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardScreen {
    pub fn new() -> Self {
        Self {
            total_balance: 0,
            wallet_count: 0,
            transaction_count: 0,
            network_stats: NetworkStats::default(),
        }
    }

    pub fn update_stats(
        &mut self,
        total_balance: u64,
        wallet_count: usize,
        transaction_count: usize,
        network_stats: NetworkStats,
    ) {
        self.total_balance = total_balance;
        self.wallet_count = wallet_count;
        self.transaction_count = transaction_count;
        self.network_stats = network_stats;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, vim_mode: &VimMode) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Main content
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Overview stats
                Constraint::Length(6), // Quick actions
                Constraint::Min(8),    // Recent activity
            ])
            .split(main_chunks[0]);

        // Overview stats
        self.render_overview(frame, chunks[0]);

        // Quick actions
        self.render_quick_actions(frame, chunks[1]);

        // Recent activity (placeholder)
        self.render_recent_activity(frame, chunks[2]);

        // Status bar
        let mut status_bar = StatusBarComponent::new();
        status_bar.update_network_stats(self.network_stats.clone());
        status_bar.set_current_screen("Dashboard".to_string());
        status_bar.set_vim_mode(vim_mode.clone());
        status_bar.render(frame, main_chunks[1]);
    }

    fn render_overview(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(area);

        // Total Balance
        let balance_text = vec![
            Line::from(vec![
                Span::styled("💰", AppStyles::highlighted()),
                Span::raw(" Total Balance"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format_balance(self.total_balance),
                if self.total_balance > 0 {
                    AppStyles::balance_positive()
                } else {
                    AppStyles::balance_zero()
                },
            )]),
        ];

        let balance_block = Paragraph::new(balance_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Balance")
                .title_style(AppStyles::title())
                .border_style(AppStyles::border()),
        );

        frame.render_widget(balance_block, chunks[0]);

        // Wallet Count
        let wallet_text = vec![
            Line::from(vec![
                Span::styled("🗂️", AppStyles::highlighted()),
                Span::raw(" Wallets"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                self.wallet_count.to_string(),
                AppStyles::info(),
            )]),
        ];

        let wallet_block = Paragraph::new(wallet_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Wallets")
                .title_style(AppStyles::title())
                .border_style(AppStyles::border()),
        );

        frame.render_widget(wallet_block, chunks[1]);

        // Transaction Count
        let tx_text = vec![
            Line::from(vec![
                Span::styled("📤", AppStyles::highlighted()),
                Span::raw(" Transactions"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                self.transaction_count.to_string(),
                AppStyles::info(),
            )]),
        ];

        let tx_block = Paragraph::new(tx_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Transactions")
                .title_style(AppStyles::title())
                .border_style(AppStyles::border()),
        );

        frame.render_widget(tx_block, chunks[2]);

        // Network Status
        let network_text = vec![
            Line::from(vec![
                Span::styled("🌐", AppStyles::highlighted()),
                Span::raw(" Network"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("{} peers", self.network_stats.connected_peers),
                if self.network_stats.connected_peers > 0 {
                    AppStyles::status_active()
                } else {
                    AppStyles::status_inactive()
                },
            )]),
            Line::from(vec![Span::styled(
                format!("Block: {}", self.network_stats.block_height),
                AppStyles::normal(),
            )]),
        ];

        let network_block = Paragraph::new(network_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Network")
                .title_style(AppStyles::title())
                .border_style(AppStyles::border()),
        );

        frame.render_widget(network_block, chunks[3]);
    }

    fn render_quick_actions(&self, frame: &mut Frame, area: Rect) {
        let actions = [
            "📤 Send Transaction (s)",
            "🗂️ Create Wallet (n)",
            "🔄 Refresh Data (r)",
            "⚙️ Settings",
        ];

        let items: Vec<ListItem> = actions
            .iter()
            .map(|action| ListItem::new(Line::from(*action)))
            .collect();

        let actions_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("⚡ Quick Actions")
                    .title_style(AppStyles::title())
                    .border_style(AppStyles::border()),
            )
            .style(AppStyles::normal());

        frame.render_widget(actions_list, area);
    }

    fn render_recent_activity(&self, frame: &mut Frame, area: Rect) {
        let activity_items = if self.transaction_count == 0 {
            vec![ListItem::new("No recent activity")]
        } else {
            vec![
                ListItem::new("✓ Blockchain synchronized"),
                ListItem::new("📤 Recent transactions loaded"),
                ListItem::new("🌐 Connected to network"),
            ]
        };

        let activity_list = List::new(activity_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📋 Recent Activity")
                    .title_style(AppStyles::title())
                    .border_style(AppStyles::border()),
            )
            .style(AppStyles::normal());

        frame.render_widget(activity_list, area);
    }
}
