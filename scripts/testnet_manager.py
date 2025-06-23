#!/usr/bin/env python3
"""
PolyTorus Local Testnet Manager
A Python script to manage local testnet operations
"""

import json
import time
import argparse
import subprocess
import requests
from typing import Dict, List, Optional
import os
import sys

class PolyTorusTestnet:
    def __init__(self):
        self.api_base = "http://localhost:9020"
        self.nodes = {
            "bootstrap": "http://localhost:9000",
            "miner-1": "http://localhost:9001", 
            "miner-2": "http://localhost:9002",
            "validator": "http://localhost:9003",
            "api-gateway": "http://localhost:9020"
        }
        
    def check_node_status(self, node_url: str) -> bool:
        """Check if a node is responsive"""
        try:
            response = requests.get(f"{node_url}/status", timeout=5)
            return response.status_code == 200
        except:
            return False
    
    def get_network_status(self) -> Dict:
        """Get overall network status"""
        status = {}
        for name, url in self.nodes.items():
            status[name] = {
                "online": self.check_node_status(url),
                "url": url
            }
        return status
    
    def create_wallet(self) -> Optional[Dict]:
        """Create a new wallet"""
        try:
            response = requests.post(f"{self.api_base}/wallet/create")
            if response.status_code == 200:
                return response.json()
            else:
                print(f"Failed to create wallet: HTTP {response.status_code}")
                return None
        except Exception as e:
            print(f"Error creating wallet: {e}")
            return None
    
    def list_wallets(self) -> List[Dict]:
        """List all available wallets"""
        try:
            response = requests.get(f"{self.api_base}/wallet/list")
            if response.status_code == 200:
                return response.json()
            else:
                return []
        except Exception as e:
            print(f"Error listing wallets: {e}")
            return []
    
    def get_balance(self, address: str) -> Optional[float]:
        """Get balance for an address"""
        try:
            response = requests.get(f"{self.api_base}/balance/{address}")
            if response.status_code == 200:
                data = response.json()
                return data.get('balance', 0)
            else:
                return None
        except Exception as e:
            print(f"Error getting balance: {e}")
            return None
    
    def send_transaction(self, from_addr: str, to_addr: str, amount: float, gas_price: int = 1) -> Optional[str]:
        """Send a transaction"""
        try:
            payload = {
                "from": from_addr,
                "to": to_addr,
                "amount": amount,
                "gasPrice": gas_price
            }
            response = requests.post(f"{self.api_base}/transaction/send", json=payload)
            if response.status_code == 200:
                data = response.json()
                return data.get('hash')
            else:
                print(f"Failed to send transaction: HTTP {response.status_code}")
                return None
        except Exception as e:
            print(f"Error sending transaction: {e}")
            return None
    
    def get_recent_transactions(self) -> List[Dict]:
        """Get recent transactions"""
        try:
            response = requests.get(f"{self.api_base}/transaction/recent")
            if response.status_code == 200:
                return response.json()
            else:
                return []
        except Exception as e:
            print(f"Error getting transactions: {e}")
            return []
    
    def get_blockchain_stats(self) -> Optional[Dict]:
        """Get blockchain statistics"""
        try:
            response = requests.get(f"{self.api_base}/network/status")
            if response.status_code == 200:
                return response.json()
            else:
                return None
        except Exception as e:
            print(f"Error getting blockchain stats: {e}")
            return None

def print_status(testnet: PolyTorusTestnet):
    """Print network status"""
    print("🌐 PolyTorus Local Testnet Status")
    print("=" * 40)
    
    status = testnet.get_network_status()
    for name, info in status.items():
        status_icon = "✅" if info["online"] else "❌"
        print(f"{status_icon} {name.capitalize()}: {info['url']}")
    
    print("\n📊 Blockchain Statistics")
    print("-" * 25)
    stats = testnet.get_blockchain_stats()
    if stats:
        print(f"Block Height: {stats.get('blockHeight', 'N/A')}")
        print(f"Total Transactions: {stats.get('totalTransactions', 'N/A')}")
        print(f"Difficulty: {stats.get('difficulty', 'N/A')}")
    else:
        print("Unable to fetch blockchain statistics")

def interactive_mode(testnet: PolyTorusTestnet):
    """Interactive command mode"""
    print("🎮 PolyTorus Interactive Mode")
    print("Type 'help' for available commands, 'quit' to exit")
    
    while True:
        try:
            command = input("\npolytest> ").strip().lower()
            
            if command == 'quit' or command == 'exit':
                break
            elif command == 'help':
                print("""
Available commands:
  status          - Show network status
  wallets         - List all wallets  
  create-wallet   - Create a new wallet
  balance <addr>  - Get balance for address
  send <from> <to> <amount> - Send transaction
  transactions    - Show recent transactions
  stats           - Show blockchain statistics
  help            - Show this help
  quit/exit       - Exit interactive mode
                """)
            elif command == 'status':
                print_status(testnet)
            elif command == 'wallets':
                wallets = testnet.list_wallets()
                if wallets:
                    print("\n👛 Available Wallets:")
                    for i, wallet in enumerate(wallets, 1):
                        print(f"{i}. {wallet['address']} ({wallet.get('type', 'unknown')})")
                else:
                    print("No wallets found. Create one with 'create-wallet'")
            elif command == 'create-wallet':
                wallet = testnet.create_wallet()
                if wallet:
                    print(f"✅ New wallet created: {wallet['address']}")
                else:
                    print("❌ Failed to create wallet")
            elif command.startswith('balance '):
                parts = command.split()
                if len(parts) == 2:
                    address = parts[1]
                    balance = testnet.get_balance(address)
                    if balance is not None:
                        print(f"💰 Balance: {balance} POLY")
                    else:
                        print("❌ Failed to get balance")
                else:
                    print("Usage: balance <address>")
            elif command.startswith('send '):
                parts = command.split()
                if len(parts) >= 4:
                    from_addr = parts[1]
                    to_addr = parts[2]
                    try:
                        amount = float(parts[3])
                        tx_hash = testnet.send_transaction(from_addr, to_addr, amount)
                        if tx_hash:
                            print(f"✅ Transaction sent: {tx_hash}")
                        else:
                            print("❌ Failed to send transaction")
                    except ValueError:
                        print("❌ Invalid amount")
                else:
                    print("Usage: send <from_address> <to_address> <amount>")
            elif command == 'transactions':
                txs = testnet.get_recent_transactions()
                if txs:
                    print("\n📋 Recent Transactions:")
                    for tx in txs[-10:]:  # Show last 10
                        print(f"  {tx['hash'][:16]}... {tx['from'][:8]}→{tx['to'][:8]} {tx['amount']} POLY")
                else:
                    print("No recent transactions")
            elif command == 'stats':
                stats = testnet.get_blockchain_stats()
                if stats:
                    print(f"\n📊 Blockchain Statistics:")
                    print(f"Block Height: {stats.get('blockHeight', 'N/A')}")
                    print(f"Total Transactions: {stats.get('totalTransactions', 'N/A')}")
                    print(f"Difficulty: {stats.get('difficulty', 'N/A')}")
                else:
                    print("❌ Unable to fetch statistics")
            elif command == '':
                continue
            else:
                print(f"Unknown command: {command}. Type 'help' for available commands.")
                
        except KeyboardInterrupt:
            print("\nExiting...")
            break
        except Exception as e:
            print(f"Error: {e}")

def send_test_transactions(testnet: PolyTorusTestnet, count: int = 5):
    """Send test transactions automatically"""
    print(f"🔄 Sending {count} test transactions...")
    
    wallets = testnet.list_wallets()
    if len(wallets) < 2:
        print("❌ Need at least 2 wallets for test transactions")
        return
    
    sent = 0
    for i in range(count):
        from_wallet = wallets[i % len(wallets)]
        to_wallet = wallets[(i + 1) % len(wallets)]
        amount = 1.0 + (i * 0.1)  # Varying amounts
        
        tx_hash = testnet.send_transaction(from_wallet['address'], to_wallet['address'], amount)
        if tx_hash:
            print(f"✅ Transaction {i+1}/{count}: {tx_hash[:16]}...")
            sent += 1
        else:
            print(f"❌ Failed to send transaction {i+1}")
        
        time.sleep(2)  # Wait between transactions
    
    print(f"✅ Sent {sent}/{count} test transactions successfully")

def main():
    parser = argparse.ArgumentParser(description="PolyTorus Local Testnet Manager")
    parser.add_argument('--status', action='store_true', help='Show network status')
    parser.add_argument('--interactive', '-i', action='store_true', help='Start interactive mode')
    parser.add_argument('--test-transactions', type=int, metavar='COUNT', help='Send test transactions')
    parser.add_argument('--create-wallet', action='store_true', help='Create a new wallet')
    parser.add_argument('--list-wallets', action='store_true', help='List all wallets')
    parser.add_argument('--balance', metavar='ADDRESS', help='Get balance for address')
    
    args = parser.parse_args()
    
    testnet = PolyTorusTestnet()
    
    if args.status:
        print_status(testnet)
    elif args.interactive:
        interactive_mode(testnet)
    elif args.test_transactions:
        send_test_transactions(testnet, args.test_transactions)
    elif args.create_wallet:
        wallet = testnet.create_wallet()
        if wallet:
            print(f"✅ New wallet created: {wallet['address']}")
        else:
            print("❌ Failed to create wallet")
    elif args.list_wallets:
        wallets = testnet.list_wallets()
        if wallets:
            print("👛 Available Wallets:")
            for i, wallet in enumerate(wallets, 1):
                print(f"{i}. {wallet['address']} ({wallet.get('type', 'unknown')})")
        else:
            print("No wallets found")
    elif args.balance:
        balance = testnet.get_balance(args.balance)
        if balance is not None:
            print(f"💰 Balance: {balance} POLY")
        else:
            print("❌ Failed to get balance")
    else:
        print("PolyTorus Local Testnet Manager")
        print("Use --help for available options")
        print("Quick start: python3 testnet_manager.py --interactive")

if __name__ == "__main__":
    main()