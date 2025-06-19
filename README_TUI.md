# Polytorus TUI - Terminal User Interface

A beautiful and powerful Terminal User Interface for the Polytorus blockchain platform, built with `ratatui`.

## Features

### 🎨 Modern UI Design
- **Multiple Screens**: Dashboard, Wallets, Transactions, Network
- **Responsive Layout**: Adapts to terminal size
- **Color-coded Interface**: Visual feedback for different states
- **Keyboard Navigation**: Vim-style and arrow key support

### 💰 Transaction Focus
- **Interactive Transaction Form**: Step-by-step transaction creation
- **Real-time Validation**: Address and amount validation
- **Balance Checking**: Insufficient balance detection  
- **Transaction History**: View sent and received transactions
- **Status Tracking**: Pending, confirmed, and failed states

### 🗂️ Wallet Management
- **Multiple Wallets**: Support for multiple wallet addresses
- **Balance Display**: Real-time balance updates
- **Wallet Creation**: Create new ECDSA wallets
- **Address Management**: Easy address selection and copying

### 🌐 Network Information
- **Network Status**: Connection and sync status
- **Peer Information**: Connected peers and network health
- **Block Height**: Current blockchain height
- **Hash Rate**: Network hash rate display

## Quick Start

### Build and Run
```bash
# Build the TUI binary
cargo build --bin polytorus_tui

# Run the TUI application
./target/debug/polytorus_tui
```

### Keyboard Shortcuts

#### Global Navigation
- `1-4` - Switch between screens (Dashboard, Wallets, Transactions, Network)
- `Tab` / `Shift+Tab` - Navigate between panels
- `↑↓` / `j k` - Navigate lists
- `Enter` - Select / Confirm
- `Esc` - Close popup / Cancel
- `q` / `Ctrl+C` - Quit application

#### Wallet Actions
- `s` - Send transaction (when wallet selected)
- `n` - Create new wallet
- `r` - Refresh data

#### Help
- `?` / `h` - Show help popup

#### Transaction Form
- `Tab` / `Shift+Tab` - Navigate form fields
- `Type` - Enter address/amount
- `Backspace` - Delete character
- `Enter` - Send transaction (on confirm button)

## Screen Overview

### 📊 Dashboard Screen
- **Overview Statistics**: Total balance, wallet count, transaction count
- **Network Status**: Connection status and block height
- **Quick Actions**: Common operations at a glance
- **Recent Activity**: Latest blockchain events

### 💰 Wallets Screen
- **Wallet List**: All available wallets with balances
- **Wallet Details**: Selected wallet information
- **Balance Display**: BTC and satoshi amounts
- **Address Management**: Easy wallet selection

### 📤 Transactions Screen
- **Transaction History**: Complete transaction list
- **Transaction Details**: Hash, amount, addresses, timestamps
- **Status Indicators**: Visual confirmation status
- **Real-time Updates**: Live transaction status updates

### 🌐 Network Screen
- **Network Overview**: Connection and sync status
- **Peer List**: Connected peers with statistics
- **Network Actions**: Connection and sync controls
- **Health Monitoring**: Network performance metrics

## Architecture

### Component Structure
```
src/tui/
├── app.rs              # Main application logic
├── components/         # Reusable UI components
│   ├── wallet_list.rs     # Wallet list component
│   ├── transaction_form.rs # Transaction form overlay
│   ├── transaction_list.rs # Transaction history
│   ├── status_bar.rs      # Bottom status bar
│   └── help_popup.rs      # Help overlay
├── screens/            # Full-screen views
│   ├── dashboard.rs       # Overview screen
│   ├── wallets.rs         # Wallet management
│   ├── transactions.rs    # Transaction history
│   └── network.rs         # Network information
├── styles.rs           # Color and style definitions
└── utils.rs            # Helper functions and types
```

### Integration Points
- **Wallet Backend**: Integrates with existing `crypto::wallets::Wallets`
- **Blockchain**: Uses `UnifiedModularOrchestrator` for blockchain operations
- **Configuration**: Respects existing `DataContext` and configuration
- **Networking**: Displays real network status and peer information

## Customization

### Styling
The TUI uses a consistent color scheme defined in `styles.rs`:
- **Primary**: Cyan for titles and highlights
- **Success**: Green for positive states
- **Warning**: Yellow for caution states
- **Error**: Red for error states
- **Info**: Blue for informational text

### Configuration
The TUI respects all existing Polytorus configuration:
- Data directories from `DataContext`
- Network settings from configuration files
- Wallet encryption types and preferences

## Development

### Adding New Screens
1. Create new screen module in `src/tui/screens/`
2. Implement the screen with `render()` method
3. Add to the main application router in `app.rs`
4. Add keyboard shortcut for navigation

### Adding New Components
1. Create component in `src/tui/components/`
2. Implement with `render()` method taking `Frame` and `Rect`
3. Add to the appropriate screen
4. Export in the module's `mod.rs`

### Extending Functionality
- **Real Transaction Sending**: Implement actual transaction creation and signing
- **Live Updates**: Add periodic blockchain state refreshing
- **Settings Screen**: Add configuration management
- **Advanced Features**: Smart contracts, governance, mining

## Dependencies

- **ratatui**: Terminal UI framework
- **crossterm**: Cross-platform terminal handling  
- **tokio**: Async runtime for blockchain integration
- **chrono**: Date and time formatting
- **anyhow**: Error handling

## Examples

### Send Transaction Flow
1. Navigate to Wallets screen (`2`)
2. Select a wallet with balance (arrow keys)
3. Press `s` to open transaction form
4. Fill in recipient address (Tab to navigate fields)
5. Enter amount in BTC
6. Navigate to Send button and press Enter
7. Transaction is created and added to history

### Create New Wallet
1. Press `n` from any screen
2. New ECDSA wallet is created automatically
3. Address is added to wallet list
4. Wallet is saved to disk

### View Network Status
1. Navigate to Network screen (`4`)
2. View connection status and peer count
3. Monitor blockchain synchronization
4. Check network health metrics

## Future Enhancements

- **Smart Contract Interface**: Deploy and interact with contracts
- **Mining Dashboard**: Mining status and controls
- **Governance Interface**: Proposal creation and voting
- **Multi-signature Support**: Multi-sig wallet management
- **Hardware Wallet**: Hardware wallet integration
- **QR Code Support**: QR code generation and scanning
- **Export/Import**: Transaction and wallet data export