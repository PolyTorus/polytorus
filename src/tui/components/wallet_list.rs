//! Wallet list component

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::tui::{
    styles::AppStyles,
    utils::{format_address, format_balance, WalletInfo},
};

#[derive(Clone)]
pub struct WalletListComponent {
    pub wallets: Vec<WalletInfo>,
    pub state: ListState,
}

impl Default for WalletListComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl WalletListComponent {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            wallets: Vec::new(),
            state,
        }
    }

    pub fn with_wallets(mut self, wallets: Vec<WalletInfo>) -> Self {
        self.wallets = wallets;
        if !self.wallets.is_empty() && self.state.selected().is_none() {
            self.state.select(Some(0));
        }
        self
    }

    pub fn add_wallet(&mut self, wallet: WalletInfo) {
        self.wallets.push(wallet);
        if self.state.selected().is_none() {
            self.state.select(Some(0));
        }
    }

    pub fn selected_wallet(&self) -> Option<&WalletInfo> {
        self.state.selected().and_then(|i| self.wallets.get(i))
    }

    pub fn next(&mut self) {
        if self.wallets.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.wallets.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.wallets.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.wallets.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(area);

        // Wallet list
        let border_style = if focused {
            AppStyles::border_focused()
        } else {
            AppStyles::border()
        };

        let items: Vec<ListItem> = self
            .wallets
            .iter()
            .map(|wallet| {
                let balance_text = format_balance(wallet.balance);
                let address_text = format_address(&wallet.address, 40);

                let balance_style = if wallet.balance > 0 {
                    AppStyles::balance_positive()
                } else {
                    AppStyles::balance_zero()
                };

                let label = if let Some(ref label) = wallet.label {
                    format!("{} ({})", label, address_text)
                } else {
                    address_text
                };

                ListItem::new(vec![Line::from(vec![
                    Span::styled("💰 ", AppStyles::highlighted()),
                    Span::raw(label),
                    Span::raw(" - "),
                    Span::styled(balance_text, balance_style),
                ])])
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("💰 Wallets")
                    .title_style(AppStyles::title())
                    .border_style(border_style),
            )
            .highlight_style(AppStyles::selected())
            .highlight_symbol("➤ ");

        frame.render_stateful_widget(list, chunks[0], &mut self.state);

        // Selected wallet details
        if let Some(wallet) = self.selected_wallet() {
            let details = vec![
                Line::from(vec![
                    Span::styled("Address: ", AppStyles::info()),
                    Span::raw(&wallet.address),
                ]),
                Line::from(vec![
                    Span::styled("Balance: ", AppStyles::info()),
                    Span::styled(
                        format_balance(wallet.balance),
                        if wallet.balance > 0 {
                            AppStyles::balance_positive()
                        } else {
                            AppStyles::balance_zero()
                        },
                    ),
                ]),
            ];

            let details_paragraph = Paragraph::new(details).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📊 Wallet Details")
                    .title_style(AppStyles::title())
                    .border_style(border_style),
            );

            frame.render_widget(details_paragraph, chunks[1]);
        } else {
            let no_wallet = Paragraph::new("No wallet selected")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("📊 Wallet Details")
                        .title_style(AppStyles::title())
                        .border_style(border_style),
                )
                .style(AppStyles::warning());

            frame.render_widget(no_wallet, chunks[1]);
        }
    }
}
