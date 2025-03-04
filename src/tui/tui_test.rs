#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::*;
    use ratatui::*;
    use std::io;
    use std::time::*;

    #[test]
    fn test_tui_print_chain() {
        tui_print_chain();
    }

    #[test]
    fn test_tui_create_wallet() {
        tui_create_wallet();
    }

    #[test]
    fn test_tui_get_balance() {
        tui_get_balance();
    }

    #[test]
    fn test_tui_run_app() {
        tui_run_app();
    }

    #[test]
    fn tui_combine_test() {
        tui_print_chain();
        tui_create_wallet();
        tui_get_balance();
        tui_run_app();
    }
}