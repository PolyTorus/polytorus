//! Status bar component

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::{
    styles::AppStyles,
    utils::NetworkStats,
    vim_mode::{get_mode_indicator, VimMode},
};

pub struct StatusBarComponent {
    pub network_stats: NetworkStats,
    pub current_screen: String,
    pub vim_mode: VimMode,
}

impl Default for StatusBarComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusBarComponent {
    pub fn new() -> Self {
        Self {
            network_stats: NetworkStats::default(),
            current_screen: "Dashboard".to_string(),
            vim_mode: VimMode::Normal,
        }
    }

    pub fn update_network_stats(&mut self, stats: NetworkStats) {
        self.network_stats = stats;
    }

    pub fn set_current_screen(&mut self, screen: String) {
        self.current_screen = screen;
    }

    pub fn set_vim_mode(&mut self, mode: VimMode) {
        self.vim_mode = mode;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(20), // Current screen
                Constraint::Min(10),    // Network status
                Constraint::Length(15), // Block height
                Constraint::Length(12), // Peers
                Constraint::Length(20), // Sync status
                Constraint::Length(15), // Vim mode
            ])
            .split(area);

        // Current screen
        let screen_text = format!("📍 {}", self.current_screen);
        let screen_paragraph = Paragraph::new(screen_text)
            .style(AppStyles::info())
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::RIGHT)
                    .border_style(AppStyles::border()),
            );
        frame.render_widget(screen_paragraph, chunks[0]);

        // Network status
        let (status_text, status_style) = if self.network_stats.connected_peers > 0 {
            ("🌐 Connected", AppStyles::status_active())
        } else {
            ("🌐 Disconnected", AppStyles::status_inactive())
        };

        let network_paragraph = Paragraph::new(status_text)
            .style(status_style)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::RIGHT)
                    .border_style(AppStyles::border()),
            );
        frame.render_widget(network_paragraph, chunks[1]);

        // Block height
        let block_text = format!("🔗 {}", self.network_stats.block_height);
        let block_paragraph = Paragraph::new(block_text)
            .style(AppStyles::normal())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::RIGHT)
                    .border_style(AppStyles::border()),
            );
        frame.render_widget(block_paragraph, chunks[2]);

        // Connected peers
        let peers_text = format!("👥 {}", self.network_stats.connected_peers);
        let peers_paragraph = Paragraph::new(peers_text)
            .style(AppStyles::normal())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::RIGHT)
                    .border_style(AppStyles::border()),
            );
        frame.render_widget(peers_paragraph, chunks[3]);

        // Sync status
        let (sync_text, sync_style) = if self.network_stats.is_syncing {
            ("⏳ Syncing...", AppStyles::warning())
        } else {
            ("✓ Synchronized", AppStyles::success())
        };

        let sync_paragraph = Paragraph::new(sync_text)
            .style(sync_style)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::RIGHT)
                    .border_style(AppStyles::border()),
            );
        frame.render_widget(sync_paragraph, chunks[4]);

        // Vim mode
        let mode_text = get_mode_indicator(&self.vim_mode);
        let mode_display = if mode_text.is_empty() {
            "NORMAL".to_string()
        } else {
            mode_text.to_string()
        };

        let mode_paragraph = Paragraph::new(mode_display)
            .style(match self.vim_mode {
                VimMode::Normal => AppStyles::normal(),
                VimMode::Insert => AppStyles::success(),
                VimMode::Command => AppStyles::warning(),
                VimMode::Visual => AppStyles::highlighted(),
            })
            .alignment(Alignment::Center);
        frame.render_widget(mode_paragraph, chunks[5]);
    }
}
