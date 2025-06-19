//! Main TUI Application

use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Frame, Terminal,
};
use tokio::time::Duration;

use crate::{
    config::DataContext,
    crypto::{types::EncryptionType, wallets::Wallets},
    modular::{default_modular_config, UnifiedModularOrchestrator},
    tui::{
        components::{HelpPopupComponent, TransactionFormComponent},
        screens::{DashboardScreen, NetworkScreen, TransactionsScreen, WalletsScreen},
        utils::{NetworkStats, TransactionInfo, TransactionStatus, WalletInfo},
        vim_mode::{get_mode_indicator, VimAction, VimCommandParser, VimKeybindings, VimMode},
    },
    Result,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    Dashboard,
    Wallets,
    Transactions,
    Network,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Normal,
    SendTransaction,
    Help,
    Command,
}

pub struct TuiApp {
    // Application state
    pub current_screen: AppScreen,
    pub app_state: AppState,
    pub should_quit: bool,

    // Vim mode state
    pub vim_mode: VimMode,
    pub command_buffer: String,

    // Screens
    pub dashboard_screen: DashboardScreen,
    pub wallets_screen: WalletsScreen,
    pub transactions_screen: TransactionsScreen,
    pub network_screen: NetworkScreen,

    // Components
    pub transaction_form: TransactionFormComponent,

    // Backend integration
    pub orchestrator: Option<UnifiedModularOrchestrator>,
    pub wallets: Option<Wallets>,
    pub data_context: DataContext,

    // State
    pub network_stats: NetworkStats,
}

impl TuiApp {
    pub async fn new() -> Result<Self> {
        let data_context = DataContext::default();
        data_context.ensure_directories()?;

        Ok(Self {
            current_screen: AppScreen::Dashboard,
            app_state: AppState::Normal,
            should_quit: false,
            vim_mode: VimMode::Normal,
            command_buffer: String::new(),
            dashboard_screen: DashboardScreen::new(),
            wallets_screen: WalletsScreen::new(),
            transactions_screen: TransactionsScreen::new(),
            network_screen: NetworkScreen::new(),
            transaction_form: TransactionFormComponent::new(),
            orchestrator: None,
            wallets: None,
            data_context,
            network_stats: NetworkStats::default(),
        })
    }

    pub async fn initialize_backend(&mut self) -> Result<()> {
        // Initialize wallets
        let wallets = Wallets::new_with_context(self.data_context.clone())?;

        // Load wallet information
        let wallet_addresses = wallets.get_all_addresses();
        let mut wallet_infos = Vec::new();

        for (i, address) in wallet_addresses.iter().enumerate() {
            // For now, use placeholder balance - in real implementation,
            // this would query the blockchain
            let balance = if i == 0 { 150000000 } else { 0 }; // 1.5 BTC for first wallet
            let wallet_info =
                WalletInfo::new(address.clone(), balance).with_label(format!("Wallet {}", i + 1));
            wallet_infos.push(wallet_info);
        }

        self.wallets_screen = self
            .wallets_screen
            .clone()
            .with_wallets(wallet_infos.clone());

        // Calculate total balance
        let total_balance: u64 = wallet_infos.iter().map(|w| w.balance).sum();

        // Initialize orchestrator
        let config = default_modular_config();
        let orchestrator = UnifiedModularOrchestrator::create_and_start_with_defaults(
            config,
            self.data_context.clone(),
        )
        .await?;

        // Get network stats
        let state = orchestrator.get_state().await;
        self.network_stats = NetworkStats {
            connected_peers: 3, // Simulated
            block_height: state.current_block_height,
            is_syncing: false,
            network_hash_rate: "1.2 TH/s".to_string(),
        };

        // Update dashboard
        self.dashboard_screen.update_stats(
            total_balance,
            wallet_infos.len(),
            0, // Transaction count - would be loaded from blockchain
            self.network_stats.clone(),
        );

        // Store the backend
        self.orchestrator = Some(orchestrator);
        self.wallets = Some(wallets);

        // Create some sample transactions for demo
        let sample_transactions = vec![
            TransactionInfo {
                hash: "0x1234567890abcdef...".to_string(),
                from: wallet_addresses
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "N/A".to_string()),
                to: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
                amount: 50000000, // 0.5 BTC
                timestamp: "2024-01-15 14:30:00".to_string(),
                status: TransactionStatus::Confirmed,
            },
            TransactionInfo {
                hash: "0xabcdef1234567890...".to_string(),
                from: "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq".to_string(),
                to: wallet_addresses
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "N/A".to_string()),
                amount: 100000000, // 1.0 BTC
                timestamp: "2024-01-14 10:15:00".to_string(),
                status: TransactionStatus::Confirmed,
            },
        ];

        self.transactions_screen = self
            .transactions_screen
            .clone()
            .with_transactions(sample_transactions);

        Ok(())
    }

    pub async fn run() -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Create and run app
        let mut app = TuiApp::new().await?;
        app.initialize_backend().await?;

        let result = app.run_app(&mut terminal).await;

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            // Handle events with timeout
            if let Ok(event) = event::poll(Duration::from_millis(100)) {
                if event {
                    if let Ok(Event::Key(key)) = event::read() {
                        if key.kind == KeyEventKind::Press {
                            self.handle_key_event(key).await?;
                        }
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        match self.app_state {
            AppState::Normal => match self.current_screen {
                AppScreen::Dashboard => {
                    self.dashboard_screen.render(frame, area, &self.vim_mode);
                }
                AppScreen::Wallets => {
                    self.wallets_screen
                        .render(frame, area, true, &self.vim_mode);
                }
                AppScreen::Transactions => {
                    self.transactions_screen
                        .render(frame, area, true, &self.vim_mode);
                }
                AppScreen::Network => {
                    self.network_screen.render(frame, area, &self.vim_mode);
                }
            },
            AppState::SendTransaction => {
                // Render the current screen as background
                match self.current_screen {
                    AppScreen::Dashboard => {
                        self.dashboard_screen.render(frame, area, &self.vim_mode)
                    }
                    AppScreen::Wallets => {
                        self.wallets_screen
                            .render(frame, area, false, &self.vim_mode)
                    }
                    AppScreen::Transactions => {
                        self.transactions_screen
                            .render(frame, area, false, &self.vim_mode)
                    }
                    AppScreen::Network => self.network_screen.render(frame, area, &self.vim_mode),
                }

                // Render transaction form overlay
                self.transaction_form.render(frame, area);
            }
            AppState::Help => {
                // Render the current screen as background
                match self.current_screen {
                    AppScreen::Dashboard => {
                        self.dashboard_screen.render(frame, area, &self.vim_mode)
                    }
                    AppScreen::Wallets => {
                        self.wallets_screen
                            .render(frame, area, false, &self.vim_mode)
                    }
                    AppScreen::Transactions => {
                        self.transactions_screen
                            .render(frame, area, false, &self.vim_mode)
                    }
                    AppScreen::Network => self.network_screen.render(frame, area, &self.vim_mode),
                }

                // Render help overlay
                HelpPopupComponent::render(frame, area);
            }
            AppState::Command => {
                // Render the current screen as background
                match self.current_screen {
                    AppScreen::Dashboard => {
                        self.dashboard_screen.render(frame, area, &self.vim_mode)
                    }
                    AppScreen::Wallets => {
                        self.wallets_screen
                            .render(frame, area, false, &self.vim_mode)
                    }
                    AppScreen::Transactions => {
                        self.transactions_screen
                            .render(frame, area, false, &self.vim_mode)
                    }
                    AppScreen::Network => self.network_screen.render(frame, area, &self.vim_mode),
                }

                // Render command line at bottom
                self.render_command_line(frame, area);
            }
        }
    }

    async fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        // Use vim-style keybinding handler
        let action = VimKeybindings::handle_key(self.vim_mode.clone(), key);
        self.handle_vim_action(action).await
    }

    async fn handle_vim_action(&mut self, action: VimAction) -> Result<()> {
        match action {
            VimAction::Quit => {
                self.should_quit = true;
            }
            VimAction::MoveUp => match self.current_screen {
                AppScreen::Wallets => self.wallets_screen.previous_wallet(),
                AppScreen::Transactions => self.transactions_screen.previous_transaction(),
                _ => {}
            },
            VimAction::MoveDown => match self.current_screen {
                AppScreen::Wallets => self.wallets_screen.next_wallet(),
                AppScreen::Transactions => self.transactions_screen.next_transaction(),
                _ => {}
            },
            VimAction::NextTab => {
                self.next_screen();
            }
            VimAction::PrevTab => {
                self.previous_screen();
            }
            VimAction::SendTransaction => {
                if let Some(wallet) = self.wallets_screen.selected_wallet() {
                    self.transaction_form = TransactionFormComponent::new()
                        .with_from_address(wallet.address.clone(), wallet.balance);
                    self.app_state = AppState::SendTransaction;
                    self.vim_mode = VimMode::Insert;
                }
            }
            VimAction::NewWallet => {
                self.create_new_wallet().await?;
            }
            VimAction::Refresh => {
                self.refresh_data().await?;
            }
            VimAction::Help => {
                self.app_state = AppState::Help;
            }
            VimAction::Select => {
                if self.app_state == AppState::SendTransaction {
                    if self.transaction_form.current_field
                        == crate::tui::components::transaction_form::FormField::Confirm
                    {
                        self.handle_transaction_send().await?;
                    } else {
                        self.transaction_form.next_field();
                    }
                }
            }
            VimAction::EnterInsert => {
                if self.app_state == AppState::Normal {
                    if let Some(_wallet) = self.wallets_screen.selected_wallet() {
                        self.vim_mode = VimMode::Insert;
                        // Could start inline editing here
                    }
                }
            }
            VimAction::EnterCommand => {
                self.app_state = AppState::Command;
                self.vim_mode = VimMode::Command;
                self.command_buffer.clear();
            }
            VimAction::EnterVisual => {
                self.vim_mode = VimMode::Visual;
            }
            VimAction::ExitMode => {
                match self.app_state {
                    AppState::SendTransaction => {
                        self.app_state = AppState::Normal;
                        self.transaction_form.clear();
                    }
                    AppState::Help => {
                        self.app_state = AppState::Normal;
                    }
                    AppState::Command => {
                        self.app_state = AppState::Normal;
                        self.command_buffer.clear();
                    }
                    _ => {}
                }
                self.vim_mode = VimMode::Normal;
            }
            VimAction::InputChar(c) => match self.app_state {
                AppState::SendTransaction => {
                    self.transaction_form.input_char(c);
                }
                AppState::Command => {
                    self.command_buffer.push(c);
                }
                _ => {}
            },
            VimAction::DeleteChar => match self.app_state {
                AppState::SendTransaction => {
                    self.transaction_form.delete_char();
                }
                AppState::Command => {
                    self.command_buffer.pop();
                }
                _ => {}
            },
            VimAction::Confirm => {
                match self.app_state {
                    AppState::SendTransaction => {
                        if self.transaction_form.current_field
                            == crate::tui::components::transaction_form::FormField::Confirm
                        {
                            self.handle_transaction_send().await?;
                        } else {
                            self.transaction_form.next_field();
                        }
                    }
                    AppState::Command => {
                        let command = self.command_buffer.clone();
                        let command_action = VimCommandParser::parse_command(&command);
                        self.app_state = AppState::Normal;
                        self.vim_mode = VimMode::Normal;
                        self.command_buffer.clear();

                        // Handle command actions directly to avoid recursion
                        match command_action {
                            VimAction::Quit => self.should_quit = true,
                            VimAction::NewWallet => {
                                self.create_new_wallet().await?;
                            }
                            VimAction::Refresh => {
                                self.refresh_data().await?;
                            }
                            VimAction::SendTransaction => {
                                if let Some(wallet) = self.wallets_screen.selected_wallet() {
                                    self.transaction_form = TransactionFormComponent::new()
                                        .with_from_address(wallet.address.clone(), wallet.balance);
                                    self.app_state = AppState::SendTransaction;
                                    self.vim_mode = VimMode::Insert;
                                }
                            }
                            VimAction::ExecuteCommand(cmd) => match cmd.as_str() {
                                "goto_dashboard" => self.current_screen = AppScreen::Dashboard,
                                "goto_wallets" => self.current_screen = AppScreen::Wallets,
                                "goto_transactions" => {
                                    self.current_screen = AppScreen::Transactions
                                }
                                "goto_network" => self.current_screen = AppScreen::Network,
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            VimAction::ExecuteCommand(cmd) => match cmd.as_str() {
                "goto_dashboard" => self.current_screen = AppScreen::Dashboard,
                "goto_wallets" => self.current_screen = AppScreen::Wallets,
                "goto_transactions" => self.current_screen = AppScreen::Transactions,
                "goto_network" => self.current_screen = AppScreen::Network,
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn next_screen(&mut self) {
        self.current_screen = match self.current_screen {
            AppScreen::Dashboard => AppScreen::Wallets,
            AppScreen::Wallets => AppScreen::Transactions,
            AppScreen::Transactions => AppScreen::Network,
            AppScreen::Network => AppScreen::Dashboard,
        };
    }

    fn previous_screen(&mut self) {
        self.current_screen = match self.current_screen {
            AppScreen::Dashboard => AppScreen::Network,
            AppScreen::Wallets => AppScreen::Dashboard,
            AppScreen::Transactions => AppScreen::Wallets,
            AppScreen::Network => AppScreen::Transactions,
        };
    }

    fn render_command_line(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            text::{Line, Span},
            widgets::{Block, Borders, Paragraph},
        };

        use crate::tui::styles::AppStyles;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(area);

        let command_text = format!(":{}", self.command_buffer);
        let mode_text = get_mode_indicator(&self.vim_mode);

        let command_paragraph = Paragraph::new(vec![
            Line::from(vec![Span::styled(command_text, AppStyles::input_focused())]),
            Line::from(vec![Span::styled(mode_text, AppStyles::info())]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Command Mode")
                .title_style(AppStyles::title())
                .border_style(AppStyles::border_focused()),
        );

        frame.render_widget(command_paragraph, chunks[1]);
    }

    async fn handle_transaction_send(&mut self) -> Result<()> {
        match self.transaction_form.validate() {
            Ok((from, to, amount)) => {
                match self.send_transaction(from, to, amount).await {
                    Ok(tx_hash) => {
                        self.transaction_form
                            .set_success(format!("Transaction sent! Hash: {}", tx_hash));

                        // Add to transaction list
                        let new_tx = TransactionInfo {
                            hash: tx_hash,
                            from: self.transaction_form.from_address.clone(),
                            to: self.transaction_form.to_address.clone(),
                            amount,
                            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            status: TransactionStatus::Pending,
                        };
                        self.transactions_screen.add_transaction(new_tx);

                        // Clear form after successful send
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        self.transaction_form.clear();
                        self.app_state = AppState::Normal;
                        self.vim_mode = VimMode::Normal;
                    }
                    Err(e) => {
                        self.transaction_form
                            .set_error(format!("Transaction failed: {}", e));
                    }
                }
            }
            Err(e) => {
                self.transaction_form.set_error(e);
            }
        }
        Ok(())
    }

    async fn create_new_wallet(&mut self) -> Result<()> {
        if let Some(ref mut wallets) = self.wallets {
            let address = wallets.create_wallet(EncryptionType::ECDSA);
            wallets.save_all()?;

            let wallet_info = WalletInfo::new(address, 0)
                .with_label(format!("Wallet {}", wallets.get_all_addresses().len()));

            self.wallets_screen.add_wallet(wallet_info);
        }
        Ok(())
    }

    async fn send_transaction(&self, _from: String, _to: String, _amount: u64) -> Result<String> {
        // In a real implementation, this would:
        // 1. Create and sign the transaction
        // 2. Submit it to the orchestrator
        // 3. Return the transaction hash

        // For demo purposes, generate a mock transaction hash
        let tx_hash = format!("0x{:016x}", rand::random::<u64>());
        Ok(tx_hash)
    }

    async fn refresh_data(&mut self) -> Result<()> {
        // Refresh network stats
        if let Some(ref orchestrator) = self.orchestrator {
            let state = orchestrator.get_state().await;
            self.network_stats.block_height = state.current_block_height;

            // Update all screens with new network stats
            self.dashboard_screen.network_stats = self.network_stats.clone();
            self.wallets_screen
                .update_network_stats(self.network_stats.clone());
            self.transactions_screen
                .update_network_stats(self.network_stats.clone());
            self.network_screen
                .update_network_stats(self.network_stats.clone());
        }
        Ok(())
    }
}
