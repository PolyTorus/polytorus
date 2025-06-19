//! Vim-style mode and keybinding management

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum VimMode {
    Normal,
    Insert,
    Command,
    Visual,
}

#[derive(Debug, Clone)]
pub enum VimAction {
    // Navigation
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveToTop,
    MoveToBottom,
    MovePageUp,
    MovePageDown,

    // Screen navigation
    NextTab,
    PrevTab,

    // Mode changes
    EnterInsert,
    EnterCommand,
    EnterVisual,
    ExitMode,

    // Actions
    Select,
    Confirm,
    Cancel,
    Refresh,
    NewWallet,
    SendTransaction,
    Help,
    Quit,

    // Command mode
    ExecuteCommand(String),

    // Input
    InputChar(char),
    DeleteChar,

    // No action
    None,
}

pub struct VimKeybindings;

impl VimKeybindings {
    pub fn handle_key(mode: VimMode, key: KeyEvent) -> VimAction {
        match mode {
            VimMode::Normal => Self::handle_normal_mode(key),
            VimMode::Insert => Self::handle_insert_mode(key),
            VimMode::Command => Self::handle_command_mode(key),
            VimMode::Visual => Self::handle_visual_mode(key),
        }
    }

    fn handle_normal_mode(key: KeyEvent) -> VimAction {
        match key.code {
            // Quit
            KeyCode::Char('q') => VimAction::Quit,
            KeyCode::Char('Q') => VimAction::Quit,

            // Navigation - vim style
            KeyCode::Char('h') => VimAction::MoveLeft,
            KeyCode::Char('j') => VimAction::MoveDown,
            KeyCode::Char('k') => VimAction::MoveUp,
            KeyCode::Char('l') => VimAction::MoveRight,

            // Navigation - alternative
            KeyCode::Up => VimAction::MoveUp,
            KeyCode::Down => VimAction::MoveDown,
            KeyCode::Left => VimAction::MoveLeft,
            KeyCode::Right => VimAction::MoveRight,

            // Page navigation
            KeyCode::Char('g') => VimAction::MoveToTop,
            KeyCode::Char('G') => VimAction::MoveToBottom,
            KeyCode::PageUp => VimAction::MovePageUp,
            KeyCode::PageDown => VimAction::MovePageDown,
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                VimAction::MovePageUp
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                VimAction::MovePageDown
            }

            // Tab navigation
            KeyCode::Char('1') => VimAction::ExecuteCommand("goto_dashboard".to_string()),
            KeyCode::Char('2') => VimAction::ExecuteCommand("goto_wallets".to_string()),
            KeyCode::Char('3') => VimAction::ExecuteCommand("goto_transactions".to_string()),
            KeyCode::Char('4') => VimAction::ExecuteCommand("goto_network".to_string()),
            KeyCode::Tab => VimAction::NextTab,
            KeyCode::BackTab => VimAction::PrevTab,

            // Actions
            KeyCode::Enter => VimAction::Select,
            KeyCode::Char(' ') => VimAction::Select, // Space for selection
            KeyCode::Char('r') => VimAction::Refresh,
            KeyCode::Char('n') => VimAction::NewWallet,
            KeyCode::Char('s') => VimAction::SendTransaction,
            KeyCode::Char('?') => VimAction::Help,

            // Mode changes
            KeyCode::Char('i') => VimAction::EnterInsert,
            KeyCode::Char('I') => VimAction::EnterInsert,
            KeyCode::Char('a') => VimAction::EnterInsert,
            KeyCode::Char('A') => VimAction::EnterInsert,
            KeyCode::Char('o') => VimAction::EnterInsert,
            KeyCode::Char('O') => VimAction::EnterInsert,
            KeyCode::Char(':') => VimAction::EnterCommand,
            KeyCode::Char('v') => VimAction::EnterVisual,
            KeyCode::Char('V') => VimAction::EnterVisual,

            // Exit/Cancel
            KeyCode::Esc => VimAction::ExitMode,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => VimAction::Quit,

            _ => VimAction::None,
        }
    }

    fn handle_insert_mode(key: KeyEvent) -> VimAction {
        match key.code {
            KeyCode::Esc => VimAction::ExitMode,
            KeyCode::Enter => VimAction::Confirm,
            KeyCode::Tab => VimAction::NextTab,
            KeyCode::BackTab => VimAction::PrevTab,
            KeyCode::Backspace => VimAction::DeleteChar,
            KeyCode::Char(c) => VimAction::InputChar(c),
            _ => VimAction::None,
        }
    }

    fn handle_command_mode(key: KeyEvent) -> VimAction {
        match key.code {
            KeyCode::Esc => VimAction::ExitMode,
            KeyCode::Enter => VimAction::Confirm, // Will execute command
            KeyCode::Backspace => VimAction::DeleteChar,
            KeyCode::Char(c) => VimAction::InputChar(c),
            _ => VimAction::None,
        }
    }

    fn handle_visual_mode(key: KeyEvent) -> VimAction {
        match key.code {
            KeyCode::Esc => VimAction::ExitMode,

            // Navigation in visual mode
            KeyCode::Char('h') => VimAction::MoveLeft,
            KeyCode::Char('j') => VimAction::MoveDown,
            KeyCode::Char('k') => VimAction::MoveUp,
            KeyCode::Char('l') => VimAction::MoveRight,
            KeyCode::Up => VimAction::MoveUp,
            KeyCode::Down => VimAction::MoveDown,
            KeyCode::Left => VimAction::MoveLeft,
            KeyCode::Right => VimAction::MoveRight,

            // Actions in visual mode
            KeyCode::Enter => VimAction::Select,
            KeyCode::Char(' ') => VimAction::Select,
            KeyCode::Char('y') => VimAction::Select, // "yank" - copy/select

            _ => VimAction::None,
        }
    }
}

pub struct VimCommandParser;

impl VimCommandParser {
    pub fn parse_command(command: &str) -> VimAction {
        let command = command.trim();

        match command {
            // Quit commands
            "q" | "quit" => VimAction::Quit,
            "q!" | "quit!" => VimAction::Quit,
            "wq" | "x" => VimAction::Quit, // Save and quit (we auto-save)

            // Navigation commands - these need custom handling in app
            "1" | "dashboard" => VimAction::ExecuteCommand("goto_dashboard".to_string()),
            "2" | "wallets" => VimAction::ExecuteCommand("goto_wallets".to_string()),
            "3" | "transactions" | "tx" => {
                VimAction::ExecuteCommand("goto_transactions".to_string())
            }
            "4" | "network" | "net" => VimAction::ExecuteCommand("goto_network".to_string()),

            // Action commands
            "refresh" | "r" => VimAction::Refresh,
            "new" | "newwallet" => VimAction::NewWallet,
            "send" | "sendtx" => VimAction::SendTransaction,
            "help" | "h" => VimAction::Help,

            // Unknown command
            _ => {
                if command.starts_with("send ") {
                    // Could parse send commands like ":send <address> <amount>"
                    VimAction::SendTransaction
                } else {
                    VimAction::None
                }
            }
        }
    }
}

pub fn get_mode_indicator(mode: &VimMode) -> &'static str {
    match mode {
        VimMode::Normal => "",
        VimMode::Insert => "-- INSERT --",
        VimMode::Command => "-- COMMAND --",
        VimMode::Visual => "-- VISUAL --",
    }
}

pub fn get_mode_help_text(mode: &VimMode) -> Vec<&'static str> {
    match mode {
        VimMode::Normal => vec![
            "h,j,k,l - Navigate",
            "1-4 - Switch tabs",
            "s - Send transaction",
            "n - New wallet",
            "r - Refresh",
            "i - Insert mode",
            ": - Command mode",
            "v - Visual mode",
            "? - Help",
            "q - Quit",
        ],
        VimMode::Insert => vec![
            "Esc - Normal mode",
            "Enter - Confirm",
            "Tab - Next field",
            "Type to input",
        ],
        VimMode::Command => vec![
            "Esc - Normal mode",
            "Enter - Execute",
            ":q - Quit",
            ":send - Send transaction",
            ":new - New wallet",
            ":refresh - Refresh data",
        ],
        VimMode::Visual => vec![
            "Esc - Normal mode",
            "h,j,k,l - Navigate",
            "Enter - Select",
            "y - Select/copy",
        ],
    }
}
