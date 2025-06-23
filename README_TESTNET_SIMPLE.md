# 🏠 PolyTorus Local Testnet (CLI版)

**シンプルで実用的なローカルブロックチェーン開発環境**

PolyTorus Local Testnetは、開発者がローカルマシンでContainerLabを使用して完全なブロックチェーンネットワークを実行できるツールです。Web UIなしのシンプルな構成で、CLI/APIベースの開発に最適化されています。

## ⚡ クイックスタート

```bash
# 1. テストネットをビルド・開始
./start-local-testnet.sh build
./start-local-testnet.sh start

# 2. 対話型CLIを使用
./start-local-testnet.sh cli

# 3. ウォレット作成とトランザクション送信
polytest> create-wallet
polytest> wallets
polytest> send <from> <to> <amount>
```

## 🎯 環境構成

### 🌐 **5ノード構成**
- **Bootstrap** (`:9000`): ジェネシスノード、ネットワークエントリーポイント
- **Miner 1** (`:9001`): アクティブマイニングノード
- **Miner 2** (`:9002`): セカンドマイニングノード
- **Validator** (`:9003`): トランザクション検証ノード
- **API Gateway** (`:9020`): REST APIアクセスポイント

### 🔧 **開発者向け機能**
- **REST API**: 完全なブロックチェーン機能をHTTP経由で提供
- **対話型CLI**: Pythonベースの高機能コマンドラインインターフェース
- **リアルタイムマイニング**: 実際のProof-of-Workコンセンサス
- **ホットリロード**: 変更が即座に反映

## 📋 前提条件

```bash
# 必要なツール
- Docker (コンテナランタイム)
- ContainerLab (ネットワークオーケストレーション)
- Python 3 (CLIツール用)
- curl (APIテスト用)

# クイックインストール (Ubuntu/Debian)
bash -c "$(curl -sL https://get.containerlab.dev)"  # ContainerLab
curl -fsSL https://get.docker.com | sh               # Docker
```

## 🚀 基本操作

### 管理コマンド

```bash
# テストネット管理
./start-local-testnet.sh start     # テストネット開始
./start-local-testnet.sh stop      # テストネット停止
./start-local-testnet.sh restart   # テストネット再起動
./start-local-testnet.sh status    # ステータス確認
./start-local-testnet.sh logs      # ログ表示

# 開発ツール
./start-local-testnet.sh build     # Dockerイメージビルド
./start-local-testnet.sh clean     # 全データクリーンアップ
./start-local-testnet.sh api       # APIエンドポイントテスト
```

### ユーザー操作

```bash
# ウォレット・トランザクション
./start-local-testnet.sh wallet    # 新しいウォレット作成
./start-local-testnet.sh send      # テストトランザクション送信
./start-local-testnet.sh cli       # 対話型CLI起動
```

## 🎮 対話型CLI

最も強力な機能は対話型CLIです：

```bash
./start-local-testnet.sh cli

# 基本操作
polytest> help                      # 全コマンド表示
polytest> status                    # ネットワーク状況
polytest> stats                     # ブロックチェーン統計

# ウォレット操作
polytest> create-wallet              # 新しいウォレット作成
polytest> wallets                    # 全ウォレット一覧
polytest> balance <address>          # 残高確認

# トランザクション操作
polytest> send <from> <to> <amount>  # トランザクション送信
polytest> transactions              # 最近のトランザクション表示

# 終了
polytest> quit
```

## 🔗 API エンドポイント

REST API (http://localhost:9020) で全機能にアクセス：

### ウォレット操作
```bash
# ウォレット作成
curl -X POST http://localhost:9020/wallet/create

# ウォレット一覧
curl http://localhost:9020/wallet/list

# 残高確認
curl http://localhost:9020/balance/<address>
```

### トランザクション操作
```bash
# トランザクション送信
curl -X POST http://localhost:9020/transaction/send \
  -H "Content-Type: application/json" \
  -d '{
    "from": "sender_address",
    "to": "recipient_address",
    "amount": 10.5,
    "gasPrice": 1
  }'

# トランザクション状況確認
curl http://localhost:9020/transaction/status/<hash>

# 最近のトランザクション
curl http://localhost:9020/transaction/recent
```

### ネットワーク情報
```bash
# ネットワーク状況
curl http://localhost:9020/network/status

# 最新ブロック
curl http://localhost:9020/block/latest

# 特定ブロック
curl http://localhost:9020/block/<hash>
```

## 📊 ネットワーク構成

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Bootstrap  │────│   Miner 1   │────│   Miner 2   │
│   :9000     │    │   :9001     │    │   :9002     │
│  (Genesis)  │    │ (Mining)    │    │ (Mining)    │
└─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
       ┌─────────────┐    ┌─────────────┐
       │  Validator  │    │API Gateway  │
       │   :9003     │    │   :9020     │
       │(Validation) │    │(REST API)   │
       └─────────────┘    └─────────────┘
```

## 🧪 開発ワークフロー

### 1. 基本的な開発フロー
```bash
# 環境起動
./start-local-testnet.sh start

# ウォレット作成
./start-local-testnet.sh cli
polytest> create-wallet
polytest> create-wallet

# トランザクション実行
polytest> wallets
polytest> send <wallet1> <wallet2> 100

# 状況確認
polytest> transactions
polytest> stats
```

### 2. API統合テスト
```bash
# APIエンドポイントテスト
./start-local-testnet.sh api

# 個別API呼び出し
curl http://localhost:9020/network/status
curl http://localhost:9020/wallet/list
```

### 3. dApp開発
```javascript
// JavaScript例
const API_BASE = 'http://localhost:9020';

// ウォレット作成
const response = await fetch(`${API_BASE}/wallet/create`, {
  method: 'POST'
});
const wallet = await response.json();

// トランザクション送信
const txResponse = await fetch(`${API_BASE}/transaction/send`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    from: wallet.address,
    to: targetAddress,
    amount: 10.5
  })
});
```

## ⚙️ 設定

### テストネット設定 (`config/testnet.toml`)
```toml
[consensus]
block_time = 10000          # 10秒
difficulty = 2              # テスト用低難易度

[testnet]
chain_id = 31337
initial_supply = 1000000000 # 10億トークン

# テスト用事前資金アカウント
[testnet.prefunded_accounts]
"test_account_1" = 1000000  # 100万トークン
"test_account_2" = 500000   # 50万トークン
```

### ネットワーク設定のカスタマイズ
- `testnet-local.yml`: ノード構成とリソース制限
- `Dockerfile.testnet`: コンテナイメージ設定
- `config/testnet.toml`: ブロックチェーンパラメータ

## 🔧 トラブルシューティング

### 一般的な問題

**コンテナが起動しない？**
```bash
# 依存関係確認
containerlab version
docker --version

# ログ確認
./start-local-testnet.sh logs
```

**API呼び出しが失敗する？**
```bash
# 接続性テスト
curl http://localhost:9020/health

# ネットワーク確認
docker network ls
```

**ノードが応答しない？**
```bash
# ステータス確認
./start-local-testnet.sh status

# 必要に応じて再起動
./start-local-testnet.sh restart
```

### 完全リセット
```bash
# 全データクリーンアップと再構築
./start-local-testnet.sh clean
./start-local-testnet.sh build
./start-local-testnet.sh start
```

## 📚 高度な使用法

### 自動化テスト
```bash
# 複数トランザクションの自動送信
python3 scripts/testnet_manager.py --test-transactions 50

# スクリプト統合
python3 scripts/testnet_manager.py --create-wallet
python3 scripts/testnet_manager.py --list-wallets
```

### 負荷テスト
```python
# Python例：100トランザクション送信
import requests
import time

api_base = "http://localhost:9020"

for i in range(100):
    response = requests.post(f"{api_base}/transaction/send", json={
        "from": wallet1,
        "to": wallet2, 
        "amount": 1.0 + i * 0.1
    })
    print(f"Transaction {i}: {response.status_code}")
    time.sleep(1)
```

### CI/CD統合
```yaml
# GitHub Actions例
- name: Start Testnet
  run: ./start-local-testnet.sh start

- name: Run Tests
  run: python3 tests/integration_tests.py

- name: Stop Testnet
  run: ./start-local-testnet.sh stop
```

## 📖 関連ドキュメント

- **メインドキュメント**: [README.md](README.md)
- **設定ガイド**: [CONFIGURATION.md](docs/CONFIGURATION.md)
- **API リファレンス**: [API_REFERENCE.md](docs/API_REFERENCE.md)

## 🤝 サポート

- **Issues**: [GitHub Issues](https://github.com/PolyTorus/polytorus/issues)
- **Discussions**: [GitHub Discussions](https://github.com/PolyTorus/polytorus/discussions)
- **Documentation**: [Full Documentation](https://docs.polytorus.org)

---

## 🎯 今すぐ始める！

```bash
git clone https://github.com/PolyTorus/polytorus
cd polytorus
./start-local-testnet.sh build
./start-local-testnet.sh start
./start-local-testnet.sh cli
```

シンプルで強力なローカルブロックチェーン環境をお楽しみください！ 🚀