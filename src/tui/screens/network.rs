//! Network screen

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::{
    components::StatusBarComponent, styles::AppStyles, utils::NetworkStats, vim_mode::VimMode,
};

pub struct NetworkScreen {
    pub network_stats: NetworkStats,
    pub connected_peers: Vec<String>,
}

impl Default for NetworkScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkScreen {
    pub fn new() -> Self {
        Self {
            network_stats: NetworkStats::default(),
            connected_peers: Vec::new(),
        }
    }

    pub fn update_network_stats(&mut self, stats: NetworkStats) {
        self.network_stats = stats;
    }

    pub fn update_peers(&mut self, peers: Vec<String>) {
        self.connected_peers = peers;
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
                Constraint::Length(10), // Network status
                Constraint::Min(8),     // Connected peers
            ])
            .split(main_chunks[0]);

        // Network status
        self.render_network_status(frame, chunks[0]);

        // Connected peers
        self.render_connected_peers(frame, chunks[1]);

        // Status bar
        let mut status_bar = StatusBarComponent::new();
        status_bar.update_network_stats(self.network_stats.clone());
        status_bar.set_current_screen("Network".to_string());
        status_bar.set_vim_mode(vim_mode.clone());
        status_bar.render(frame, main_chunks[1]);
    }

    fn render_network_status(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Network overview
        let network_info = vec![
            Line::from(vec![
                Span::styled("Status: ", AppStyles::info()),
                Span::styled(
                    if self.network_stats.connected_peers > 0 {
                        "Connected"
                    } else {
                        "Disconnected"
                    },
                    if self.network_stats.connected_peers > 0 {
                        AppStyles::status_active()
                    } else {
                        AppStyles::status_inactive()
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("Block Height: ", AppStyles::info()),
                Span::styled(
                    self.network_stats.block_height.to_string(),
                    AppStyles::normal(),
                ),
            ]),
            Line::from(vec![
                Span::styled("Connected Peers: ", AppStyles::info()),
                Span::styled(
                    self.network_stats.connected_peers.to_string(),
                    AppStyles::highlighted(),
                ),
            ]),
            Line::from(vec![
                Span::styled("Sync Status: ", AppStyles::info()),
                Span::styled(
                    if self.network_stats.is_syncing {
                        "Syncing..."
                    } else {
                        "Synchronized"
                    },
                    if self.network_stats.is_syncing {
                        AppStyles::warning()
                    } else {
                        AppStyles::success()
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("Hash Rate: ", AppStyles::info()),
                Span::styled(&self.network_stats.network_hash_rate, AppStyles::normal()),
            ]),
        ];

        let network_block = Paragraph::new(network_info).block(
            Block::default()
                .borders(Borders::ALL)
                .title("🌐 Network Status")
                .title_style(AppStyles::title())
                .border_style(AppStyles::border()),
        );

        frame.render_widget(network_block, chunks[0]);

        // Network actions
        let actions = [
            "🔄 Refresh Network Data",
            "🔗 Connect to Peer",
            "📊 Network Statistics",
            "⚙️ Network Settings",
        ];

        let action_items: Vec<ListItem> = actions
            .iter()
            .map(|action| ListItem::new(Line::from(*action)))
            .collect();

        let actions_list = List::new(action_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("⚡ Network Actions")
                    .title_style(AppStyles::title())
                    .border_style(AppStyles::border()),
            )
            .style(AppStyles::normal());

        frame.render_widget(actions_list, chunks[1]);
    }

    fn render_connected_peers(&self, frame: &mut Frame, area: Rect) {
        let peer_items: Vec<ListItem> = if self.connected_peers.is_empty() {
            vec![ListItem::new(Line::from(vec![Span::styled(
                "No peers connected",
                AppStyles::warning(),
            )]))]
        } else {
            self.connected_peers
                .iter()
                .enumerate()
                .map(|(i, peer)| {
                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled(format!("🔗 Peer {}: ", i + 1), AppStyles::info()),
                            Span::styled(peer, AppStyles::normal()),
                        ]),
                        Line::from(vec![
                            Span::raw("    "),
                            Span::styled("Status: ", AppStyles::info()),
                            Span::styled("Connected", AppStyles::success()),
                            Span::raw(" | "),
                            Span::styled("Latency: ", AppStyles::info()),
                            Span::styled("45ms", AppStyles::normal()),
                        ]),
                    ])
                })
                .collect()
        };

        let peers_list = List::new(peer_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("👥 Connected Peers")
                    .title_style(AppStyles::title())
                    .border_style(AppStyles::border()),
            )
            .style(AppStyles::normal());

        frame.render_widget(peers_list, area);
    }
}
