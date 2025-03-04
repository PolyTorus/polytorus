// src/cli/tui.rs
use crate::{
  blockchain::blockchain::Blockchain,
  command::cli::{
      cmd_create_blockchain, cmd_create_wallet, cmd_get_balance, cmd_list_address, cmd_reindex,
  },
  crypto::wallets::Wallets,
  Result,
};

use crossterm::{
  event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
  backend::CrosstermBackend,
  layout::{Alignment, Constraint, Direction, Layout},
  prelude::Backend,
  style::{Color, Style},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
  Terminal,
};
use std::io;
use std::time::{Duration, Instant};

pub fn tui_print_chain<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
  // ブロックチェーン全体を取得！Eter() は tip からジェネシス方向へ進むため、reverse して表示頁E??整える?E?E
  let bc = Blockchain::new()?;
  let mut blocks: Vec<_> = bc.iter().collect();
  blocks.reverse();

  // 蜷・ヶ繝ｭ繝・け縺ｮ讎りｦ・ｼ・eight, Hash蜈磯ｭ8譁・ｭ・ Prev蜈磯ｭ8譁・ｭ暦ｼ峨ｒ菴懈・
  let block_summaries: Vec<String> = blocks
      .iter()
      .map(|block| {
          let hash = block.get_hash();
          let prev = block.get_prev_hash();
          let hash_prefix = if hash.len() >= 8 { &hash[..8] } else { &hash };
          let prev_prefix = if prev.len() >= 8 { &prev[..8] } else { &prev };
          format!(
              "Height: {} | Hash: {} | Prev: {}",
              block.get_height(),
              hash_prefix,
              prev_prefix
          )
      })
      .collect();

  // リスト選択状態
  let mut list_state = ListState::default();
  list_state.select(Some(0));
  // 詳細パネルの横スクロール位置
  let mut detail_scroll_x: u16 = 0;

  loop {
      terminal.draw(|f| {
          let size = f.area();
          // 画面を左右に分割 左はブロックチェーン一覧 右は詳細画面
          let chunks = Layout::default()
              .direction(Direction::Horizontal)
              .margin(2)
              .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
              .split(size);

          // 左側のブロックチェーン一覧
          let items: Vec<ListItem> = block_summaries
              .iter()
              .map(|s| ListItem::new(s.clone()))
              .collect();
          let list = List::new(items)
              .block(Block::default().borders(Borders::ALL).title("Blockchain"))
              .highlight_style(Style::default().fg(Color::Yellow))
              .highlight_symbol(">> ");
          f.render_stateful_widget(list, chunks[0], &mut list_state);

          // 右側の選択中のブロックチェーンの詳細(デバッグ形式)
          let detail = if let Some(selected) = list_state.selected() {
              format!("{:#?}", blocks[selected])
          } else {
              "No block selected".to_string()
          };
          let detail_paragraph = Paragraph::new(detail)
              .block(
                  Block::default()
                      .borders(Borders::ALL)
                      .title("Block Details"),
              )
              .scroll((detail_scroll_x, 0));
          f.render_widget(detail_paragraph, chunks[1]);
      })?;

      if event::poll(Duration::from_millis(250))? {
          if let Event::Key(key) = event::read()? {
              match key.code {
                  KeyCode::Char('q') => break,
                  KeyCode::Down => {
                      if let Some(selected) = list_state.selected() {
                          let next = if selected >= block_summaries.len() - 1 {
                              0
                          } else {
                              selected + 1
                          };
                          list_state.select(Some(next));
                          detail_scroll_x = 0; // 選択変更時にスクロール位置をリセット
                      }
                  }
                  KeyCode::Up => {
                      if let Some(selected) = list_state.selected() {
                          let prev = if selected == 0 {
                              block_summaries.len() - 1
                          } else {
                              selected - 1
                          };
                          list_state.select(Some(prev));
                          detail_scroll_x = 0; // 選択変更時にスクロール位置をリセット
                      }
                  }
                  // 詳細パネルの横スクロール
                  KeyCode::Left => {
                      if detail_scroll_x > 0 {
                          detail_scroll_x -= 1;
                      }
                  }
                  KeyCode::Right => {
                      detail_scroll_x += 1;
                  }
                  _ => {}
              }
          }
      }
  }

  Ok(())
}

pub fn tui_create_wallet<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
  let address = cmd_create_wallet()?;
  loop {
      terminal.draw(|f| {
          let size = f.area();
          let block = Block::default()
              .borders(Borders::ALL)
              .title("Create Wallet");
          let paragraph = Paragraph::new(format!(
              "Wallet created:\n\n{}\n\nPress any key to return.",
              address
          ))
          .block(block)
          .alignment(Alignment::Center)
          .wrap(Wrap { trim: true });
          f.render_widget(paragraph, size);
      })?;
      if event::poll(Duration::from_millis(250))? {
          if let Event::Key(_) = event::read()? {
              break;
          }
      }
  }
  Ok(())
}

pub fn tui_get_balance<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
  // ウォレットの全アドレスを取得
  let ws = Wallets::new()?;
  let addresses = ws.get_all_addresses();

  if addresses.is_empty() {
      // アドレスが無い場合エラーメッセージを表示して終了
      loop {
          terminal.draw(|f| {
              let size = f.area();
              let block = Block::default().borders(Borders::ALL).title("Get Balance");
              let paragraph = Paragraph::new(
                  "No wallet found. Please create one first.\n\nPress any key to return.",
              )
              .block(block)
              .alignment(Alignment::Center)
              .wrap(Wrap { trim: true });
              f.render_widget(paragraph, size);
          })?;
          if event::poll(Duration::from_millis(250))? {
              if let Event::Key(_) = event::read()? {
                  return Ok(());
              }
          }
      }
  }

  // アドレス一覧から選択する ListState を作成
  let mut list_state = ListState::default();
  list_state.select(Some(0));

  // アドレス一覧の選択画面
  loop {
      terminal.draw(|f| {
          let size = f.area();
          let items: Vec<ListItem> = addresses
              .iter()
              .map(|a| ListItem::new(a.as_str()))
              .collect();
          let list = List::new(items)
              .block(
                  Block::default()
                      .borders(Borders::ALL)
                      .title("Select Wallet for Balance (Enter to confirm, q to cancel)"),
              )
              .highlight_style(Style::default().fg(Color::Yellow))
              .highlight_symbol(">> ");
          f.render_stateful_widget(list, size, &mut list_state);
      })?;

      if event::poll(Duration::from_millis(250))? {
          if let Event::Key(key) = event::read()? {
              match key.code {
                  KeyCode::Char('q') => return Ok(()),
                  KeyCode::Down => {
                      if let Some(selected) = list_state.selected() {
                          let next = if selected >= addresses.len() - 1 {
                              0
                          } else {
                              selected + 1
                          };
                          list_state.select(Some(next));
                      }
                  }
                  KeyCode::Up => {
                      if let Some(selected) = list_state.selected() {
                          let prev = if selected == 0 {
                              addresses.len() - 1
                          } else {
                              selected - 1
                          };
                          list_state.select(Some(prev));
                      }
                  }
                  KeyCode::Enter => break, // 選択確定
                  _ => {}
              }
          }
      }
  }

  // 選択されたアドレスを取得し，その残高を計算
  let selected_index = list_state.selected().unwrap();
  let addr = addresses[selected_index].clone();
  let balance = cmd_get_balance(&addr)?;

  // 残高結果の画面を表示
  loop {
      terminal.draw(|f| {
          let size = f.area();
          let block = Block::default()
              .borders(Borders::ALL)
              .title("Balance Result");
          let paragraph = Paragraph::new(format!(
              "Wallet: {}\nBalance: {}\n\nPress any key to return.",
              addr, balance
          ))
          .block(block)
          .alignment(Alignment::Center)
          .wrap(Wrap { trim: true });
          f.render_widget(paragraph, size);
      })?;
      if event::poll(Duration::from_millis(250))? {
          if let Event::Key(_) = event::read()? {
              break;
          }
      }
  }
  Ok(())
}

pub struct TuiApp {
  pub menu_items: Vec<&'static str>,
  pub state: ListState,
}

impl TuiApp {
  pub fn new() -> Self {
      let menu_items = vec![
          "Print Chain",
          "Create Wallet",
          "List Addresses",
          "Reindex UTXO",
          "Create Blockchain",
          "Get Balance",
          "Send",
          "Start Node",
          "Start Miner",
          "Quit",
      ];
      let mut state = ListState::default();
      state.select(Some(0));
      Self { menu_items, state }
  }

  /// 次の項目へ移動
  pub fn next(&mut self) {
      let i = match self.state.selected() {
          Some(i) => {
              if i >= self.menu_items.len() - 1 {
                  0
              } else {
                  i + 1
              }
          }
          None => 0,
      };
      self.state.select(Some(i));
  }

  /// 前の項目へ移動
  pub fn previous(&mut self) {
      let i = match self.state.selected() {
          Some(i) => {
              if i == 0 {
                  self.menu_items.len() - 1
              } else {
                  i - 1
              }
          }
          None => 0,
      };
      self.state.select(Some(i));
  }

  /// 現在選択されている項目を取得
  pub fn selected_item(&self) -> Option<&&str> {
      self.state.selected().map(|i| &self.menu_items[i])
  }
}

/// TUI エントリーポイントを開始する
pub fn run_tui() -> Result<()> {
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let res = run_app(&mut terminal);

  disable_raw_mode()?;
  execute!(
      terminal.backend_mut(),
      LeaveAlternateScreen,
      DisableMouseCapture
  )?;
  terminal.show_cursor()?;
  res
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
  let mut app = TuiApp::new();
  let tick_rate = Duration::from_millis(250);
  let mut last_tick = Instant::now();

  loop {
      terminal.draw(|f| {
          let size = f.area();
          let chunks = Layout::default()
              .direction(Direction::Vertical)
              .margin(2)
              .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
              .split(size);

          let items: Vec<ListItem> = app.menu_items.iter().map(|i| ListItem::new(*i)).collect();
          let list = List::new(items)
              .block(Block::default().borders(Borders::ALL).title("Menu"))
              .highlight_style(Style::default().fg(Color::Yellow))
              .highlight_symbol(">> ");
          f.render_stateful_widget(list, chunks[0], &mut app.state);

          let instructions = Block::default().borders(Borders::ALL).title("Instructions");
          f.render_widget(instructions, chunks[1]);
      })?;

      let timeout = tick_rate
          .checked_sub(last_tick.elapsed())
          .unwrap_or_else(|| Duration::from_secs(0));
      if event::poll(timeout)? {
          if let Event::Key(key) = event::read()? {
              match key.code {
                  KeyCode::Char('q') => break,
                  KeyCode::Down => app.next(),
                  KeyCode::Up => app.previous(),
                  KeyCode::Enter => {
                      if let Some(selected) = app.selected_item() {
                          match *selected {
                              "Print Chain" => {
                                  tui_print_chain(terminal)?;
                              }
                              "Create Wallet" => {
                                  let addr = cmd_create_wallet()?;
                                  println!("Wallet created: {}", addr);
                              }
                              "List Addresses" => {
                                  cmd_list_address()?;
                              }
                              "Reindex UTXO" => {
                                  let count = cmd_reindex()?;
                                  println!("UTXO reindexed. Transaction count: {}", count);
                              }
                              "Create Blockchain" => {
                                  let ws = Wallets::new()?;
                                  if let Some(addr) = ws.get_all_addresses().first() {
                                      cmd_create_blockchain(addr)?;
                                  } else {
                                      println!("No wallet found. Create one first.");
                                  }
                              }
                              "Get Balance" => {
                                  // let ws = Wallets::new()?;
                                  // if let Some(addr) = ws.get_all_addresses().first() {
                                  //     let balance = cmd_get_balance(addr)?;
                                  //     println!("Balance for {}: {}", addr, balance);
                                  // } else {
                                  //     println!("No wallet found. Create one first.");
                                  // }
                                  tui_get_balance(terminal)?;
                              }
                              "Send" => {
                                  // Send は対話入力が主になるためにここでは簡易的にメッセージ表示のみ
                                  println!(
                                      "Send functionality is not fully interactive in TUI yet."
                                  );
                              }
                              "Start Node" => {
                                  println!(
                                      "Start Node functionality is not supported in TUI mode."
                                  );
                              }
                              "Start Miner" => {
                                  println!(
                                      "Start Miner functionality is not supported in TUI mode."
                                  );
                              }
                              "Quit" => break,
                              _ => {}
                          }
                      }
                  }
                  _ => {}
              }
          }
      }
      if last_tick.elapsed() >= tick_rate {
          last_tick = Instant::now();
      }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    

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
