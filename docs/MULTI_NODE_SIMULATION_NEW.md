# Multi-Node Transaction Simulation & Complete Propagation

PolyTorusブロックチェーンの複数ノード環境でのトランザクションシミュレーション機能です。
**完全なトランザクション伝播**をサポートし、送信と受信の両方を正確に追跡します。

## 🎯 新機能: 完全なトランザクション伝播

### 概要
- **送信側API**: `/send`エンドポイントで送信者ノードの`tx_count`をインクリメント
- **受信側API**: `/transaction`エンドポイントで受信者ノードの`rx_count`をインクリメント  
- **完全な追跡**: 各トランザクションが送信側と受信側の両方で正しく記録される

### 伝播フロー
```
送信者ノード           受信者ノード
    ↓                     ↓
POST /send           POST /transaction
    ↓                     ↓
tx_count++           rx_count++
    ↓                     ↓
「送信記録」         「受信記録」
```

## 🚀 クイックスタート

### 方法1: 統合スクリプトを使用

```bash
# 基本的なシミュレーション（4ノード、5分間）
./scripts/simulate.sh local

# 完全な伝播テスト（推奨）
./scripts/test_complete_propagation.sh

# カスタム設定でのシミュレーション
./scripts/simulate.sh local --nodes 6 --duration 600 --interval 3000
```

### 方法2: 手動での完全伝播テスト

```bash
# Step 1: 送信者ノードに送信を記録
curl -X POST -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}' \
  "http://127.0.0.1:9000/send"

# Step 2: 受信者ノードに受信を記録  
curl -X POST -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}' \
  "http://127.0.0.1:9001/transaction"
```

### 方法3: リアルタイム監視

```bash
# トランザクション監視ツール
cargo run --example transaction_monitor

# ノード統計の確認
for port in 9000 9001 9002 9003; do
  echo "Node port $port:"; curl -s "http://127.0.0.1:$port/stats"; echo ""
done
```

## 🌐 HTTP API エンドポイント

各ノードは以下のHTTP APIを提供します：

### 完全伝播対応API

- `POST /send` - **送信記録API** (送信者ノードで使用)
- `POST /transaction` - **受信記録API** (受信者ノードで使用)
- `GET /stats` - **統計情報** (送信/受信カウンターを含む)
- `GET /status` - ノードの状態
- `GET /health` - ヘルスチェック

### API使用例

```bash
# 完全なトランザクション伝播の例：Node 0 → Node 1

# Step 1: 送信者ノード(Node 0)で送信を記録
curl -X POST http://127.0.0.1:9000/send \
  -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}'

# Step 2: 受信者ノード(Node 1)で受信を記録
curl -X POST http://127.0.0.1:9001/transaction \
  -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}'

# Step 3: 統計を確認
curl http://127.0.0.1:9000/stats  # 送信者の統計
curl http://127.0.0.1:9001/stats  # 受信者の統計
```

### レスポンス例

**送信記録API (`/send`) のレスポンス:**
```json
{
  "status": "sent",
  "transaction_id": "8d705e89-50fb-4a34-bb0e-a8083bbcb40c",
  "message": "Transaction from wallet_node-0 to wallet_node-1 for 100 sent"
}
```

**受信記録API (`/transaction`) のレスポンス:**
```json
{
  "status": "accepted", 
  "transaction_id": "baf3ecb7-86dd-4523-9d8a-0eb90eb6da43",
  "message": "Transaction from wallet_node-0 to wallet_node-1 for 100 accepted"
}
```

**統計API (`/stats`) のレスポンス:**
```json
{
  "transactions_sent": 3,
  "transactions_received": 8,
  "timestamp": "2025-06-15T19:47:44.380841660+00:00",
  "node_id": "node-0"
}
```

## 📊 監視とデバッグ

### リアルタイム監視

```bash
# 専用監視ツール (表形式で見やすく表示)
cargo run --example transaction_monitor

# シンプルな統計確認
curl -s http://127.0.0.1:9000/stats | jq '.'

# 全ノードの統計一括確認
for port in 9000 9001 9002 9003; do
  node_num=$((port - 9000))
  echo "Node $node_num: $(curl -s http://127.0.0.1:$port/stats)"
done
```

### 実行結果の例

```
📊 Network Statistics - 2025-06-15 19:47:44 UTC
┌─────────┬────────┬──────────┬──────────┬────────────┬─────────────┐
│ Node    │ Status │ TX Sent  │ TX Recv  │ Block Height│ Last Update │
├─────────┼────────┼──────────┼──────────┼────────────┼─────────────┤
│ node-0  │ 🟢 Online  │        3 │        8 │          0 │ 0s ago      │
│ node-1  │ 🟢 Online  │        1 │       19 │          0 │ 0s ago      │
│ node-2  │ 🟢 Online  │        1 │       18 │          0 │ 0s ago      │
│ node-3  │ 🟢 Online  │        1 │       10 │          0 │ 0s ago      │
├─────────┼────────┼──────────┼──────────┼────────────┼─────────────┤
│ Total   │  4/4  ON │        6 │       55 │ N/A        │ Summary     │
└─────────┴────────┴──────────┴──────────┴────────────┴─────────────┘
```

## ⚙️ 設定オプション

### シミュレーション設定

| パラメータ | デフォルト | 説明 |
|-----------|-----------|------|
| `--nodes` | 4 | ノード数 |
| `--duration` | 300 | シミュレーション時間（秒） |
| `--interval` | 5000 | トランザクション送信間隔（ミリ秒） |
| `--base-port` | 9000 | HTTP APIベースポート |
| `--p2p-port` | 8000 | P2Pネットワークベースポート |

### ノード設定

各ノードは個別の設定ファイルを持ちます：

```toml
[network]
listen_addr = "127.0.0.1:8000"
bootstrap_peers = ["127.0.0.1:8001", "127.0.0.1:8002"]
max_peers = 50

[storage]
data_dir = "./data/simulation/node-0"
max_cache_size = 1073741824

[logging]
level = "INFO"
output = "console"
```

## 📈 パフォーマンス評価

### 完全伝播の検証

```bash
# 完全伝播テストの実行
./scripts/test_complete_propagation.sh

# 期待される結果:
# - 各ノードで transactions_sent > 0
# - 各ノードで transactions_received > 0
# - 送信数と受信数の合計が一致
```

### メトリクス

- **TX Sent**: 送信トランザクション数 (**✅ 実装済み**)
- **TX Recv**: 受信トランザクション数 (**✅ 実装済み**)
- **Network Latency**: ノード間通信遅延
- **Block Propagation**: ブロック伝播時間  
- **API Response Time**: HTTP API応答時間

## 🔄 利用可能なスクリプト

### メインスクリプト

```bash
# 統合シミュレーション管理
./scripts/simulate.sh [local|docker|rust|status|stop|clean]

# 完全伝播テスト (推奨)
./scripts/test_complete_propagation.sh

# 個別ノード起動
./scripts/multi_node_simulation.sh [nodes] [base_port] [p2p_port] [duration]
```

### 監視・分析スクリプト

```bash
# リアルタイム監視
cargo run --example transaction_monitor

# 統計情報確認
for port in 9000 9001 9002 9003; do
  echo "Node $((port-9000)): $(curl -s http://127.0.0.1:$port/stats)"
done
```

## 🛠️ トラブルシューティング

### よくある問題

1. **ポート競合エラー**
   ```bash
   # 使用中のポートを確認
   netstat -tulpn | grep :9000
   
   # 別のベースポートを使用
   ./scripts/simulate.sh local --base-port 9100
   ```

2. **TX Sent が 0 のまま**
   ```bash
   # 原因: /send エンドポイントが呼ばれていない
   # 解決策: test_complete_propagation.sh を使用
   ./scripts/test_complete_propagation.sh
   ```

3. **TX Recv が 0 のまま**
   ```bash
   # 原因: /transaction エンドポイントが呼ばれていない
   # 解決策: 受信者ノードにも正しくPOSTする
   curl -X POST http://127.0.0.1:9001/transaction -d '{...}'
   ```

4. **ノードが応答しない**
   ```bash
   # ヘルスチェック
   curl http://127.0.0.1:9000/health
   
   # プロセス確認
   ./scripts/simulate.sh status
   
   # 再起動
   ./scripts/simulate.sh stop && ./scripts/simulate.sh local
   ```

### デバッグログ

```bash
# ノードログの確認
tail -f ./data/simulation/node-0.log

# 全ノードログの監視
tail -f ./data/simulation/node-*.log

# エラーログの抽出
grep -i error ./data/simulation/node-*.log
```

## 📁 ファイル構造

```
scripts/
├── simulate.sh                    # メインシミュレーション管理
├── test_complete_propagation.sh   # 完全伝播テスト
├── multi_node_simulation.sh       # 個別シミュレーション
└── analyze_tps.sh                 # パフォーマンス分析

examples/
├── multi_node_simulation.rs       # Rust実装
└── transaction_monitor.rs         # 監視ツール

data/simulation/
├── node-0/
│   ├── config.toml
│   └── data/
├── node-1/
└── ...
```

## 🎯 成功の確認方法

### 完全伝播の確認チェックリスト

1. **✅ ノード起動確認**
   ```bash
   curl http://127.0.0.1:9000/health
   ```

2. **✅ 送信記録確認**
   ```bash
   # 送信前
   curl -s http://127.0.0.1:9000/stats | jq '.transactions_sent'  # 0
   
   # 送信実行
   curl -X POST http://127.0.0.1:9000/send -d '{...}'
   
   # 送信後
   curl -s http://127.0.0.1:9000/stats | jq '.transactions_sent'  # 1
   ```

3. **✅ 受信記録確認**
   ```bash
   # 受信前
   curl -s http://127.0.0.1:9001/stats | jq '.transactions_received'
   
   # 受信実行
   curl -X POST http://127.0.0.1:9001/transaction -d '{...}'
   
   # 受信後
   curl -s http://127.0.0.1:9001/stats | jq '.transactions_received'  # +1
   ```

4. **✅ 完全伝播テスト**
   ```bash
   ./scripts/test_complete_propagation.sh
   # 結果: 全ノードで transactions_sent > 0 AND transactions_received > 0
   ```

## 📝 更新履歴

- **2025-06-15**: 完全なトランザクション伝播機能を実装
  - `/send` エンドポイント追加（送信記録用）
  - `/transaction` エンドポイント修正（受信記録用）
  - `test_complete_propagation.sh` スクリプト追加
  - TX Sent / TX Recv の両方が正常動作を確認
