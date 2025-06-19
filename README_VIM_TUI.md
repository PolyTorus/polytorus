# Polytorus Vim-Style TUI

A powerful vim-inspired Terminal User Interface for the Polytorus blockchain platform. Experience the full power of blockchain operations with familiar vim keybindings and modes.

## 🚀 Quick Start

### Launch from CLI
```bash
# Start the main CLI and launch TUI
./target/release/polytorus --tui

# Or use the standalone TUI binary
./target/release/polytorus_tui
```

## 🔧 Vim Modes & Navigation

### 📍 **Normal Mode** (Default)
The primary mode for navigation and commands.

#### **Navigation (hjkl style)**
- `h` - Move left
- `j` - Move down  
- `k` - Move up
- `l` - Move right
- `g` - Go to top of list
- `G` - Go to bottom of list
- `Ctrl+u` - Page up
- `Ctrl+d` - Page down

#### **Screen Navigation**
- `1` - Dashboard screen
- `2` - Wallets screen
- `3` - Transactions screen
- `4` - Network screen
- `Tab` - Next screen
- `Shift+Tab` - Previous screen

#### **Core Actions**
- `s` - Send transaction (when wallet selected)
- `n` - Create new wallet
- `r` - Refresh all data
- `?` - Show help
- `q` - Quit application

#### **Mode Switching**
- `i`, `a`, `o` - Enter Insert mode
- `v`, `V` - Enter Visual mode  
- `:` - Enter Command mode

### ✏️ **Insert Mode**
Active when creating transactions or editing data.

- `Esc` - Return to Normal mode
- `Enter` - Confirm action
- `Tab` / `Shift+Tab` - Navigate form fields
- `Backspace` - Delete character
- Type normally to input text

### 👁️ **Visual Mode**
For selection and visual feedback.

- `h`, `j`, `k`, `l` - Navigate while selecting
- `Enter` or `y` - Confirm selection
- `Esc` - Return to Normal mode

### ⌨️ **Command Mode**
Execute powerful commands with `:` prefix.

#### **Navigation Commands**
- `:1` or `:dashboard` - Go to Dashboard
- `:2` or `:wallets` - Go to Wallets  
- `:3` or `:transactions` - Go to Transactions
- `:4` or `:network` - Go to Network

#### **Action Commands**
- `:q` or `:quit` - Quit application
- `:q!` - Force quit
- `:wq` or `:x` - Save and quit
- `:send` - Send transaction
- `:new` or `:newwallet` - Create new wallet
- `:refresh` or `:r` - Refresh data

## 📱 Screen Overview

### 📊 **Dashboard** (`1` or `:dashboard`)
Overview of your blockchain status:
- Total balance across all wallets
- Wallet count and transaction history
- Network connection status
- Quick action shortcuts
- Recent activity feed

**Vim Commands:**
- `s` - Quick send transaction
- `n` - Create new wallet
- `r` - Refresh data

### 💰 **Wallets** (`2` or `:wallets`)
Comprehensive wallet management:
- List all wallets with balances
- Select wallets with `j`/`k` navigation
- View detailed wallet information
- Balance display in BTC and satoshis

**Vim Commands:**
- `j`/`k` - Navigate wallet list
- `s` - Send from selected wallet
- `n` - Create new wallet
- `Enter` - Select wallet
- `i` - Edit wallet (future feature)

### 📤 **Transactions** (`3` or `:transactions`)
Transaction history and monitoring:
- Complete transaction history
- Real-time status updates
- Transaction details (hash, amounts, addresses)
- Visual status indicators

**Vim Commands:**
- `j`/`k` - Navigate transaction list
- `Enter` - View transaction details
- `r` - Refresh transaction status
- `g`/`G` - First/last transaction

### 🌐 **Network** (`4` or `:network`)
Network status and peer management:
- Connected peers list
- Network health monitoring
- Blockchain synchronization status
- Network performance metrics

**Vim Commands:**
- `r` - Refresh network status
- `j`/`k` - Navigate peer list
- Future: Connect/disconnect peers

## 💸 Transaction Workflow (Vim Style)

### Quick Send (Vim-Style)
1. **Navigate to wallet**: `2` → `j`/`k` to select
2. **Start transaction**: `s` (enters Insert mode)
3. **Fill form**: Tab between fields, type address/amount
4. **Send**: Navigate to Send button with Tab, press `Enter`
5. **Return**: Automatically returns to Normal mode

### Command-Line Send
1. **Command mode**: `:` 
2. **Send command**: `send` + `Enter`
3. **Fill form**: Same as above

## 🎨 Status Bar

The bottom status bar shows:
- 📍 Current screen name
- 🌐 Network connection status  
- 🔗 Current block height
- 👥 Connected peers count
- ⏳ Sync status
- **🔥 Current Vim Mode** (NORMAL/INSERT/COMMAND/VISUAL)

Mode colors:
- `NORMAL` - Default white
- `INSERT` - Green (active editing)
- `COMMAND` - Yellow (command input)
- `VISUAL` - Cyan (selection mode)

## ⌨️ Complete Keybinding Reference

### Normal Mode Shortcuts
```
NAVIGATION:
h j k l     - Navigate (vim style)
g / G       - Top / Bottom
Ctrl+u/d    - Page up/down
1 2 3 4     - Switch screens
Tab         - Next screen

ACTIONS:
s           - Send transaction
n           - New wallet  
r           - Refresh data
?           - Help
q           - Quit

MODE SWITCH:
i a o       - Insert mode
v V         - Visual mode
:           - Command mode
Esc         - Normal mode
```

### Command Mode Reference
```
NAVIGATION:
:1          - Dashboard
:2          - Wallets
:3          - Transactions  
:4          - Network

ACTIONS:
:q          - Quit
:send       - Send transaction
:new        - New wallet
:refresh    - Refresh data
```

### Insert Mode (Transaction Form)
```
Tab         - Next field
Shift+Tab   - Previous field
Enter       - Confirm/Send
Esc         - Cancel (Normal mode)
Backspace   - Delete char
Type        - Input data
```

## 🔥 Advanced Vim Features

### Vim-Style Movement Patterns
- `5j` - Move down 5 items (future)
- `gg` - Go to first item
- `G` - Go to last item
- `/search` - Search functionality (future)

### Visual Mode Selection
- Enter visual mode with `v`
- Navigate to select items
- `y` to "yank" (copy) selection
- `Esc` to exit visual mode

### Command History
- `:`⬆️⬇️ - Browse command history (future)
- `:!!` - Repeat last command (future)

## 🛠️ Customization

### Vim Configuration (Future)
Create `~/.polytorusrc` for custom keybindings:
```vim
" Custom key mappings
map <leader>w :wallets<CR>
map <leader>s :send<CR>
map <leader>n :new<CR>

" Custom colors
colorscheme dark
```

## 💡 Tips & Tricks

### Efficiency Tips
1. **Quick Navigation**: Use `2s` to go to wallets and immediately send
2. **Batch Operations**: Use `:refresh` after multiple transactions
3. **Status Monitoring**: Keep eye on status bar for mode/network info
4. **Command Mode**: Use `:` for complex operations

### Muscle Memory
- Coming from vim? All navigation keys work as expected
- New to vim? Start with arrow keys, gradually adopt `hjkl`
- Use `?` frequently to reference commands

### Power User Shortcuts
```bash
# Quick send workflow
2      # Go to wallets
jjj    # Navigate to wallet 3
s      # Start send
# Type address and amount
Enter  # Send transaction

# Quick refresh everything
:refresh

# Quick quit
:q
```

## 🔧 Integration with CLI

The TUI integrates seamlessly with the existing Polytorus CLI:

```bash
# Launch TUI from any CLI operation
polytorus --tui

# Continue CLI operations after TUI
polytorus --listaddresses
polytorus --getbalance <address>

# Background blockchain operations
polytorus --modular-start &
polytorus --tui
```

## 🎯 Future Enhancements

### Advanced Vim Features
- [ ] Search functionality (`/` and `?`)
- [ ] Command history and completion
- [ ] Macro recording (`qq...q`)
- [ ] Multiple window support (`:split`)
- [ ] Custom key mappings
- [ ] Vim configuration file

### Enhanced Functionality  
- [ ] Smart contract interaction
- [ ] Mining dashboard
- [ ] Governance voting interface
- [ ] Multi-signature wallet support
- [ ] Hardware wallet integration
- [ ] QR code display and scanning

The Polytorus Vim-Style TUI brings the power and efficiency of vim to blockchain operations, making complex transactions and network management as intuitive as text editing. 🚀