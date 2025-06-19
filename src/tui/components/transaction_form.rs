//! Transaction form component

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::{
    styles::AppStyles,
    utils::{format_balance, validate_address, validate_amount},
};

#[derive(Debug, Clone, PartialEq)]
pub enum FormField {
    From,
    To,
    Amount,
    Confirm,
}

#[derive(Debug, Clone)]
pub struct TransactionFormComponent {
    pub from_address: String,
    pub to_address: String,
    pub amount: String,
    pub current_field: FormField,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub available_balance: u64,
}

impl Default for TransactionFormComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionFormComponent {
    pub fn new() -> Self {
        Self {
            from_address: String::new(),
            to_address: String::new(),
            amount: String::new(),
            current_field: FormField::From,
            error_message: None,
            success_message: None,
            available_balance: 0,
        }
    }

    pub fn with_from_address(mut self, address: String, balance: u64) -> Self {
        self.from_address = address;
        self.available_balance = balance;
        self.current_field = FormField::To;
        self
    }

    pub fn next_field(&mut self) {
        self.current_field = match self.current_field {
            FormField::From => FormField::To,
            FormField::To => FormField::Amount,
            FormField::Amount => FormField::Confirm,
            FormField::Confirm => FormField::To,
        };
        self.clear_messages();
    }

    pub fn previous_field(&mut self) {
        self.current_field = match self.current_field {
            FormField::From => FormField::Confirm,
            FormField::To => FormField::From,
            FormField::Amount => FormField::To,
            FormField::Confirm => FormField::Amount,
        };
        self.clear_messages();
    }

    pub fn input_char(&mut self, c: char) {
        match self.current_field {
            FormField::From => self.from_address.push(c),
            FormField::To => self.to_address.push(c),
            FormField::Amount => {
                // Only allow numeric input and decimal point
                if c.is_ascii_digit() || c == '.' {
                    self.amount.push(c);
                }
            }
            FormField::Confirm => {} // No input for confirm button
        }
        self.clear_messages();
    }

    pub fn delete_char(&mut self) {
        match self.current_field {
            FormField::From => {
                self.from_address.pop();
            }
            FormField::To => {
                self.to_address.pop();
            }
            FormField::Amount => {
                self.amount.pop();
            }
            FormField::Confirm => {} // No input for confirm button
        }
        self.clear_messages();
    }

    pub fn validate(&self) -> Result<(String, String, u64), String> {
        if self.from_address.is_empty() {
            return Err("From address is required".to_string());
        }

        if self.to_address.is_empty() {
            return Err("To address is required".to_string());
        }

        if !validate_address(&self.to_address) {
            return Err("Invalid recipient address".to_string());
        }

        if self.amount.is_empty() {
            return Err("Amount is required".to_string());
        }

        let amount_satoshis = validate_amount(&self.amount)?;

        if amount_satoshis > self.available_balance {
            return Err("Insufficient balance".to_string());
        }

        Ok((
            self.from_address.clone(),
            self.to_address.clone(),
            amount_satoshis,
        ))
    }

    pub fn clear(&mut self) {
        self.to_address.clear();
        self.amount.clear();
        self.current_field = FormField::To;
        self.error_message = None;
        self.success_message = None;
    }

    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.success_message = None;
    }

    pub fn set_success(&mut self, message: String) {
        self.success_message = Some(message);
        self.error_message = None;
    }

    fn clear_messages(&mut self) {
        self.error_message = None;
        self.success_message = None;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Clear the area
        frame.render_widget(Clear, area);

        let popup_area = centered_rect(80, 60, area);

        let block = Block::default()
            .title("📤 Send Transaction")
            .title_style(AppStyles::title())
            .borders(Borders::ALL)
            .border_style(AppStyles::border_focused());

        frame.render_widget(block, popup_area);

        let inner = popup_area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 2,
        });

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // From
                Constraint::Length(3), // To
                Constraint::Length(3), // Amount
                Constraint::Length(1), // Spacing
                Constraint::Length(3), // Available balance
                Constraint::Length(3), // Confirm button
                Constraint::Length(2), // Messages
            ])
            .split(inner);

        // From field
        let from_style = if self.current_field == FormField::From {
            AppStyles::input_focused()
        } else {
            AppStyles::input()
        };

        let from_block = Block::default()
            .title("From Address")
            .borders(Borders::ALL)
            .border_style(if self.current_field == FormField::From {
                AppStyles::border_focused()
            } else {
                AppStyles::border()
            });

        let from_text = if self.from_address.is_empty() {
            "Select a wallet first..."
        } else {
            &self.from_address
        };

        let from_paragraph = Paragraph::new(from_text)
            .block(from_block)
            .style(from_style);

        frame.render_widget(from_paragraph, chunks[0]);

        // To field
        let to_style = if self.current_field == FormField::To {
            AppStyles::input_focused()
        } else {
            AppStyles::input()
        };

        let to_block = Block::default()
            .title("To Address")
            .borders(Borders::ALL)
            .border_style(if self.current_field == FormField::To {
                AppStyles::border_focused()
            } else {
                AppStyles::border()
            });

        let to_paragraph = Paragraph::new(self.to_address.as_str())
            .block(to_block)
            .style(to_style);

        frame.render_widget(to_paragraph, chunks[1]);

        // Amount field
        let amount_style = if self.current_field == FormField::Amount {
            AppStyles::input_focused()
        } else {
            AppStyles::input()
        };

        let amount_block = Block::default()
            .title("Amount (BTC)")
            .borders(Borders::ALL)
            .border_style(if self.current_field == FormField::Amount {
                AppStyles::border_focused()
            } else {
                AppStyles::border()
            });

        let amount_paragraph = Paragraph::new(self.amount.as_str())
            .block(amount_block)
            .style(amount_style);

        frame.render_widget(amount_paragraph, chunks[2]);

        // Available balance
        let balance_text = format!("Available: {}", format_balance(self.available_balance));
        let balance_paragraph = Paragraph::new(balance_text)
            .style(AppStyles::info())
            .alignment(Alignment::Center);

        frame.render_widget(balance_paragraph, chunks[4]);

        // Confirm button
        let confirm_style = if self.current_field == FormField::Confirm {
            AppStyles::selected()
        } else {
            AppStyles::normal()
        };

        let confirm_text = if self.current_field == FormField::Confirm {
            "➤ [SEND TRANSACTION] ⬅"
        } else {
            "[SEND TRANSACTION]"
        };

        let confirm_paragraph = Paragraph::new(confirm_text)
            .style(confirm_style)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(
                if self.current_field == FormField::Confirm {
                    AppStyles::border_focused()
                } else {
                    AppStyles::border()
                },
            ));

        frame.render_widget(confirm_paragraph, chunks[5]);

        // Messages
        if let Some(ref error) = self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(AppStyles::error())
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, chunks[6]);
        } else if let Some(ref success) = self.success_message {
            let success_paragraph = Paragraph::new(success.as_str())
                .style(AppStyles::success())
                .alignment(Alignment::Center);
            frame.render_widget(success_paragraph, chunks[6]);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
