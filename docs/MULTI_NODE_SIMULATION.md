# Multi-Node Transaction Simulation

PolyTorusブロックチェーンの複数ノード環境でのトランザクションシミュレーション機能です。

## 🚀 クイックスタート

### 方法1: 統合スクリプトを使用

```bash
# 基本的なシミュレーション（4ノード、5分間）
./scripts/simulate.sh local

# カスタム設定でのシミュレーション
./scripts/simulate.sh local --nodes 6 --duration 600 --interval 3000

# Dockerを使用したシミュレーション
./scripts/simulate.sh docker

# Rustベースのシミュレーション
./scripts/simulate.sh rust --nodes 3 --duration 300
```

### 方法2: 個別実行

```bash
# シェルスクリプトベースのシミュレーション
./scripts/multi_node_simulation.sh 4 9000 8000 300

# Rustベースのシミュレーション
cargo run --example multi_node_simulation -- --nodes 4 --duration 300

# トランザクション監視ツール
cargo run --example transaction_monitor -- --nodes 4 --base-port 9000
```

### 方法3: Docker Compose

```bash
# 全ノードをDockerで起動
docker-compose up

# 特定のサービスのみ起動
docker-compose up node-0 node-1 node-2
```

## 📊 監視とデバッグ

### リアルタイム監視

```bash
# トランザクション監視ツールを起動
cargo run --example transaction_monitor

# ログファイル監視
tail -f ./data/simulation/node-*.log

# 統合スクリプトでの状況確認
./scripts/simulate.sh status
```

### API エンドポイント

各ノードは以下のHTTP APIを提供します：

- `GET /status` - ノードの状態
- `POST /transaction` - トランザクション送信
- `GET /stats` - ノード統計情報

```bash
# ノード状態確認
curl http://127.0.0.1:9000/status

# トランザクション送信
curl -X POST http://127.0.0.1:9000/transaction \
  -H "Content-Type: application/json" \
  -d '{"from":"wallet1","to":"wallet2","amount":100}'
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

### シミュレーション結果の分析

```bash
# ログファイルから統計情報を抽出
grep "Transaction" ./data/simulation/node-*.log | wc -l

# ノード間のレイテンシ測定
./scripts/analyze_performance.sh

# TPS（Transaction Per Second）計算
./scripts/calculate_tps.sh
```

### メトリクス

- **Transaction Throughput**: 秒間処理トランザクション数
- **Network Latency**: ノード間通信遅延
- **Block Propagation**: ブロック伝播時間
- **Memory Usage**: メモリ使用量
- **CPU Usage**: CPU使用率

## 🛠️ トラブルシューティング

### よくある問題

1. **ポート競合エラー**
   ```bash
   # 使用中のポートを確認
   netstat -tulpn | grep :9000
   
   # 別のベースポートを使用
   ./scripts/simulate.sh local --base-port 9100
   ```

2. **ノード起動失敗**
   ```bash
   # ログを確認
   ./scripts/simulate.sh logs
   
   # データディレクトリをクリーン
   ./scripts/simulate.sh clean
   ```

3. **トランザクション送信失敗**
   ```bash
   # ノード状態を確認
   ./scripts/simulate.sh status
   
   # APIエンドポイントを確認
   curl http://127.0.0.1:9000/status
   ```

### デバッグモード

```bash
# デバッグログレベルで実行
RUST_LOG=debug ./scripts/simulate.sh local

# 詳細な実行ログ
./scripts/simulate.sh local --nodes 2 --duration 60 2>&1 | tee simulation.log
```

## 🔧 カスタマイズ

### 独自のトランザクションパターン

`examples/multi_node_simulation.rs`を編集して、カスタムトランザクションパターンを実装できます：

```rust
// カスタムトランザクション生成ロジック
async fn generate_custom_transaction_pattern(nodes: &[NodeInstance]) -> Result<()> {
    // 独自のロジックを実装
    Ok(())
}
```

### ネットワーク障害シミュレーション

```rust
// ネットワーク分断のシミュレーション
async fn simulate_network_partition(nodes: &mut [NodeInstance]) -> Result<()> {
    // 一部のノードの接続を切断
    Ok(())
}
```

## 📚 関連ドキュメント

- [Network Architecture](../docs/NETWORK_ARCHITECTURE.md)
- [Configuration Guide](../docs/CONFIGURATION.md)
- [Development Guide](../docs/DEVELOPMENT.md)
- [API Reference](../docs/API_REFERENCE.md)

## 🤝 コントリビューション

シミュレーション機能の改善にご協力ください：

1. 新しいシミュレーションシナリオの追加
2. パフォーマンス測定ツールの改善
3. 監視ダッシュボードの実装
4. バグ修正とドキュメント改善

## 📄 ライセンス

MIT License - 詳細は[LICENSE](../LICENSE)ファイルを確認してください。
