//! Help popup component

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::styles::AppStyles;

pub struct HelpPopupComponent;

impl HelpPopupComponent {
    pub fn render(frame: &mut Frame, area: Rect) {
        // Clear the area
        frame.render_widget(Clear, area);

        let popup_area = centered_rect(80, 70, area);

        let block = Block::default()
            .title("⚙️ Help & Shortcuts")
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
                Constraint::Length(3), // Title
                Constraint::Min(10),   // Help content
                Constraint::Length(2), // Close instruction
            ])
            .split(inner);

        // Help title
        let title_text = "Polytorus TUI - Keyboard Shortcuts";
        let title_paragraph = Paragraph::new(title_text)
            .style(AppStyles::highlighted())
            .alignment(Alignment::Center);
        frame.render_widget(title_paragraph, chunks[0]);

        // Help content
        let help_items = vec![
            ListItem::new(vec![Line::from(vec![Span::styled(
                "VIM-STYLE NAVIGATION:",
                AppStyles::warning(),
            )])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled("h j k l", AppStyles::info()),
                Span::raw(" - Navigate (left, down, up, right)"),
            ])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled("g / G", AppStyles::info()),
                Span::raw(" - Go to top / bottom"),
            ])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled("Ctrl+u / Ctrl+d", AppStyles::info()),
                Span::raw(" - Page up / Page down"),
            ])]),
            ListItem::new(vec![Line::from("")]),
            ListItem::new(vec![Line::from(vec![Span::styled(
                "VIM MODES:",
                AppStyles::warning(),
            )])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled("i / a / o", AppStyles::info()),
                Span::raw(" - Enter insert mode"),
            ])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled("v / V", AppStyles::info()),
                Span::raw(" - Enter visual mode"),
            ])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled(":", AppStyles::info()),
                Span::raw(" - Enter command mode"),
            ])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled("Esc", AppStyles::info()),
                Span::raw(" - Return to normal mode"),
            ])]),
            ListItem::new(vec![Line::from("")]),
            ListItem::new(vec![Line::from(vec![Span::styled(
                "ACTIONS:",
                AppStyles::warning(),
            )])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled("s", AppStyles::info()),
                Span::raw(" - Send transaction"),
            ])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled("n", AppStyles::info()),
                Span::raw(" - Create new wallet"),
            ])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled("r", AppStyles::info()),
                Span::raw(" - Refresh data"),
            ])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled("1-4", AppStyles::info()),
                Span::raw(" - Switch screens"),
            ])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled("q", AppStyles::info()),
                Span::raw(" - Quit application"),
            ])]),
            ListItem::new(vec![Line::from("")]),
            ListItem::new(vec![Line::from(vec![Span::styled(
                "COMMAND MODE:",
                AppStyles::warning(),
            )])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled(":q", AppStyles::info()),
                Span::raw(" - Quit"),
            ])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled(":send", AppStyles::info()),
                Span::raw(" - Send transaction"),
            ])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled(":new", AppStyles::info()),
                Span::raw(" - New wallet"),
            ])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled(":refresh", AppStyles::info()),
                Span::raw(" - Refresh data"),
            ])]),
            ListItem::new(vec![Line::from(vec![
                Span::styled(":1-4", AppStyles::info()),
                Span::raw(" - Switch screens"),
            ])]),
        ];

        let help_list = List::new(help_items).style(AppStyles::normal());

        frame.render_widget(help_list, chunks[1]);

        // Close instruction
        let close_text = "Press 'Esc' or '?' to close this help";
        let close_paragraph = Paragraph::new(close_text)
            .style(AppStyles::warning())
            .alignment(Alignment::Center);
        frame.render_widget(close_paragraph, chunks[2]);
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
