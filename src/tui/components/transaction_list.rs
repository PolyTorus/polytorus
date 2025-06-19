//! Transaction list component

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::tui::{
    styles::AppStyles,
    utils::{format_address, format_balance, format_timestamp, TransactionInfo, TransactionStatus},
};

#[derive(Clone)]
pub struct TransactionListComponent {
    pub transactions: Vec<TransactionInfo>,
    pub state: ListState,
}

impl Default for TransactionListComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionListComponent {
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
            state: ListState::default(),
        }
    }

    pub fn with_transactions(mut self, transactions: Vec<TransactionInfo>) -> Self {
        self.transactions = transactions;
        if !self.transactions.is_empty() && self.state.selected().is_none() {
            self.state.select(Some(0));
        }
        self
    }

    pub fn add_transaction(&mut self, transaction: TransactionInfo) {
        self.transactions.insert(0, transaction); // Add to front for latest first
        if self.state.selected().is_none() {
            self.state.select(Some(0));
        }
    }

    pub fn update_transaction_status(&mut self, hash: &str, status: TransactionStatus) {
        if let Some(tx) = self.transactions.iter_mut().find(|tx| tx.hash == hash) {
            tx.status = status;
        }
    }

    pub fn selected_transaction(&self) -> Option<&TransactionInfo> {
        self.state.selected().and_then(|i| self.transactions.get(i))
    }

    pub fn next(&mut self) {
        if self.transactions.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.transactions.len() - 1 {
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
        if self.transactions.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.transactions.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            AppStyles::border_focused()
        } else {
            AppStyles::border()
        };

        if self.transactions.is_empty() {
            let empty_list = List::new(vec![ListItem::new("No transactions found")])
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("📤 Recent Transactions")
                        .title_style(AppStyles::title())
                        .border_style(border_style),
                )
                .style(AppStyles::warning());

            frame.render_widget(empty_list, area);
            return;
        }

        let items: Vec<ListItem> = self
            .transactions
            .iter()
            .map(|tx| {
                let status_style = match tx.status {
                    TransactionStatus::Confirmed => AppStyles::success(),
                    TransactionStatus::Pending => AppStyles::warning(),
                    TransactionStatus::Failed => AppStyles::error(),
                };

                let status_symbol = match tx.status {
                    TransactionStatus::Confirmed => "✓",
                    TransactionStatus::Pending => "⏳",
                    TransactionStatus::Failed => "✗",
                };

                let amount_text = format_balance(tx.amount);
                let from_text = format_address(&tx.from, 15);
                let to_text = format_address(&tx.to, 15);
                let time_text = format_timestamp(&tx.timestamp);

                // Determine transaction direction style
                let direction_style = AppStyles::transaction_sent(); // Default to sent
                let direction_symbol = "→";

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(format!("{} ", status_symbol), status_style),
                        Span::styled(format!("{} ", direction_symbol), direction_style),
                        Span::styled(amount_text, AppStyles::highlighted()),
                        Span::raw(format!(" | {} → {}", from_text, to_text)),
                    ]),
                    Line::from(vec![
                        Span::raw("    "),
                        Span::styled(
                            format!("Hash: {}", format_address(&tx.hash, 20)),
                            AppStyles::info(),
                        ),
                        Span::raw(" | "),
                        Span::styled(time_text, AppStyles::normal()),
                        Span::raw(" | "),
                        Span::styled(tx.status.to_string(), status_style),
                    ]),
                ])
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("📤 Recent Transactions")
                    .title_style(AppStyles::title())
                    .border_style(border_style),
            )
            .highlight_style(AppStyles::selected())
            .highlight_symbol("➤ ");

        frame.render_stateful_widget(list, area, &mut self.state);
    }
}
