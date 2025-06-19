//! Style definitions for the TUI

use ratatui::{
    style::{Color, Modifier, Style},
    symbols,
};

pub struct AppStyles;

impl AppStyles {
    pub fn normal() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn selected() -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Color::LightCyan)
            .add_modifier(Modifier::BOLD)
    }

    pub fn highlighted() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn title() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn border_focused() -> Style {
        Style::default().fg(Color::Cyan)
    }

    pub fn success() -> Style {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    }

    pub fn error() -> Style {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    }

    pub fn warning() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn info() -> Style {
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    }

    pub fn input() -> Style {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    }

    pub fn input_focused() -> Style {
        Style::default().fg(Color::White).bg(Color::Blue)
    }

    pub fn header() -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Gray)
            .add_modifier(Modifier::BOLD)
    }

    pub fn balance_positive() -> Style {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    }

    pub fn balance_zero() -> Style {
        Style::default().fg(Color::Gray)
    }

    pub fn transaction_sent() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn transaction_received() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn status_active() -> Style {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    }

    pub fn status_inactive() -> Style {
        Style::default().fg(Color::Red)
    }
}

pub struct AppSymbols;

impl AppSymbols {
    pub const BLOCK: &'static str = symbols::block::FULL;
    pub const DOT: &'static str = "•";
    pub const ARROW_RIGHT: &'static str = "→";
    pub const ARROW_LEFT: &'static str = "←";
    pub const ARROW_UP: &'static str = "↑";
    pub const ARROW_DOWN: &'static str = "↓";
    pub const CHECKMARK: &'static str = "✓";
    pub const CROSS: &'static str = "✗";
    pub const WALLET: &'static str = "💰";
    pub const TRANSACTION: &'static str = "📤";
    pub const BLOCKCHAIN: &'static str = "🔗";
    pub const NETWORK: &'static str = "🌐";
    pub const SETTINGS: &'static str = "⚙️";
}
