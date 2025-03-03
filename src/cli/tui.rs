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
  // 繝悶Ο繝・け繝√ぉ繝ｼ繝ｳ蜈ｨ菴薙ｒ蜿門ｾ暦ｼ・ter() 縺ｯ tip 縺九ｉ繧ｸ繧ｧ繝阪す繧ｹ譁ｹ蜷代∈騾ｲ繧縺溘ａ縲〉everse 縺励※陦ｨ遉ｺ鬆・ｒ謨ｴ縺医ｋ・・
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

  // 繝ｪ繧ｹ繝磯∈謚樒憾諷・
  let mut list_state = ListState::default();
  list_state.select(Some(0));
  // 隧ｳ邏ｰ繝代ロ繝ｫ縺ｮ讓ｪ繧ｹ繧ｯ繝ｭ繝ｼ繝ｫ菴咲ｽｮ
  let mut detail_scroll_x: u16 = 0;

  loop {
      terminal.draw(|f| {
          let size = f.area();
          // 逕ｻ髱｢繧貞ｷｦ蜿ｳ縺ｫ蛻・牡・壼ｷｦ縺ｯ繝悶Ο繝・け荳隕ｧ縲∝承縺ｯ隧ｳ邏ｰ諠・ｱ
          let chunks = Layout::default()
              .direction(Direction::Horizontal)
              .margin(2)
              .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
              .split(size);

          // 蟾ｦ蛛ｴ・壹ヶ繝ｭ繝・け荳隕ｧ
          let items: Vec<ListItem> = block_summaries
              .iter()
              .map(|s| ListItem::new(s.clone()))
              .collect();
          let list = List::new(items)
              .block(Block::default().borders(Borders::ALL).title("Blockchain"))
              .highlight_style(Style::default().fg(Color::Yellow))
              .highlight_symbol(">> ");
          f.render_stateful_widget(list, chunks[0], &mut list_state);

          // 蜿ｳ蛛ｴ・夐∈謚樔ｸｭ繝悶Ο繝・け縺ｮ隧ｳ邏ｰ・医ョ繝舌ャ繧ｰ蠖｢蠑擾ｼ・
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
                          detail_scroll_x = 0; // 驕ｸ謚槫､画峩譎ゅ・繧ｹ繧ｯ繝ｭ繝ｼ繝ｫ菴咲ｽｮ繧偵Μ繧ｻ繝・ヨ
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
                          detail_scroll_x = 0; // 驕ｸ謚槫､画峩譎ゅ・繧ｹ繧ｯ繝ｭ繝ｼ繝ｫ菴咲ｽｮ繧偵Μ繧ｻ繝・ヨ
                      }
                  }
                  // 隧ｳ邏ｰ繝代ロ繝ｫ縺ｮ讓ｪ繧ｹ繧ｯ繝ｭ繝ｼ繝ｫ
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
  // 繧ｦ繧ｩ繝ｬ繝・ヨ蜀・・蜈ｨ繧｢繝峨Ξ繧ｹ繧貞叙蠕・
  let ws = Wallets::new()?;
  let addresses = ws.get_all_addresses();

  if addresses.is_empty() {
      // 繧｢繝峨Ξ繧ｹ縺檎┌縺・ｴ蜷医・繧ｨ繝ｩ繝ｼ繝｡繝・そ繝ｼ繧ｸ繧定｡ｨ遉ｺ縺励※邨ゆｺ・
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

  // 繧｢繝峨Ξ繧ｹ荳隕ｧ縺九ｉ驕ｸ謚槭☆繧・ListState 繧剃ｽ懈・
  let mut list_state = ListState::default();
  list_state.select(Some(0));

  // 繧｢繝峨Ξ繧ｹ荳隕ｧ縺ｮ驕ｸ謚樒判髱｢
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
                  KeyCode::Enter => break, // 驕ｸ謚樒｢ｺ螳・
                  _ => {}
              }
          }
      }
  }

  // 驕ｸ謚槭＆繧後◆繧｢繝峨Ξ繧ｹ繧貞叙蠕励＠縲√◎縺ｮ谿矩ｫ倥ｒ險育ｮ・
  let selected_index = list_state.selected().unwrap();
  let addr = addresses[selected_index].clone();
  let balance = cmd_get_balance(&addr)?;

  // 谿矩ｫ倡ｵ先棡縺ｮ逕ｻ髱｢繧定｡ｨ遉ｺ
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

  /// 谺｡縺ｮ鬆・岼縺ｸ遘ｻ蜍・
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

  /// 蜑阪・鬆・岼縺ｸ遘ｻ蜍・
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

  /// 迴ｾ蝨ｨ驕ｸ謚槭＆繧後※縺・ｋ鬆・岼繧貞叙蠕・
  pub fn selected_item(&self) -> Option<&&str> {
      self.state.selected().map(|i| &self.menu_items[i])
  }
}

/// TUI 繧帝幕蟋九☆繧九お繝ｳ繝医Μ繝昴う繝ｳ繝・
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
                                  // 窶ｻ Send 縺ｯ蟇ｾ隧ｱ蜈･蜉帙′蠢・ｦ√↓縺ｪ繧九◆繧√√％縺薙〒縺ｯ邁｡譏鍋噪縺ｫ繝｡繝・そ繝ｼ繧ｸ陦ｨ遉ｺ縺ｮ縺ｿ
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
