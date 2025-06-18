# PolyTorus Testnet Deployment Guide

本ドキュメントは、PolyTorusブロックチェーンのテストネット展開に関する包括的なガイドです。

## 📋 目次

1. [現在の実装状況](#現在の実装状況)
2. [テストネット準備状況](#テストネット準備状況)
3. [即座に利用可能な展開方法](#即座に利用可能な展開方法)
4. [プライベートテストネット展開手順](#プライベートテストネット展開手順)
5. [パブリックテストネットに向けた追加実装](#パブリックテストネットに向けた追加実装)
6. [トラブルシューティング](#トラブルシューティング)

## 🎯 現在の実装状況

### ✅ 完全実装済み

**コア機能:**
- **✅ Consensus Layer**: 完全なPoW実装（6つの包括的テスト）
- **✅ Data Availability Layer**: Merkle証明システム（15の包括的テスト）
- **✅ Settlement Layer**: 不正証明付きOptimistic Rollup（13のテスト）
- **✅ P2P Network**: 高度なメッセージ優先度システム
- **✅ Smart Contracts**: WASM実行エンジン（ERC20サポート）
- **✅ CLI Tools**: 完全なコマンドラインインターフェース
- **✅ Docker Infrastructure**: マルチステージビルド対応

**展開インフラ:**
- **✅ Docker Compose**: 開発・本番環境対応
- **✅ Monitoring**: Prometheus + Grafana統合
- **✅ Load Balancing**: Nginx + SSL設定
- **✅ Database**: PostgreSQL + Redis統合

### ⚠️ 部分実装

**改善が必要な機能:**
- **⚠️ Execution Layer**: 単体テストが不足
- **⚠️ Unified Orchestrator**: 統合テストが不足
- **⚠️ Genesis Block**: 自動生成機能なし
- **⚠️ Validator Management**: ステーキング機能制限

## 🚀 テストネット準備状況

### 現在利用可能な展開レベル

| 展開タイプ | 準備状況 | 推奨ノード数 | セキュリティレベル |
|-----------|---------|-------------|------------------|
| **ローカル開発** | ✅ 100% | 1-10 | 開発用 |
| **プライベートコンソーシアム** | ✅ 90% | 4-50 | 内部テスト |
| **パブリックテストネット** | ⚠️ 65% | 100+ | 要追加実装 |

## 🔧 即座に利用可能な展開方法

### 1. クイックスタート（ローカル）

```bash
# 1. プロジェクトのビルド
cargo build --release

# 2. 単一ノードの起動
./target/release/polytorus --modular-start --http-port 9000

# 3. ウォレット作成
./target/release/polytorus --createwallet

# 4. ステータス確認
./target/release/polytorus --modular-status
```

### 2. マルチノードシミュレーション

```bash
# 4ノードローカルネットワーク
./scripts/simulate.sh local --nodes 4 --duration 300

# Rustベースのマルチノードテスト
cargo run --example multi_node_simulation

# P2P特化テスト
cargo run --example p2p_multi_node_simulation
```

### 3. Docker展開

```bash
# 基本4ノード構成
docker-compose up

# 開発環境（監視付き）
docker-compose -f docker-compose.dev.yml up

# 本番環境設定
docker-compose -f docker-compose.prod.yml up
```

## 🏗️ プライベートテストネット展開手順

### 前提条件

**システム要件:**
- OS: Linux (Ubuntu 20.04+ 推奨)
- RAM: 8GB以上
- Storage: 100GB以上
- CPU: 4コア以上

**依存関係:**
```bash
# Rust (1.82+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# OpenFHE
sudo ./scripts/install_openfhe.sh

# Docker & Docker Compose
sudo apt-get update
sudo apt-get install docker.io docker-compose

# 環境変数設定
export OPENFHE_ROOT=/usr/local
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH
```

### Step 1: プロジェクトセットアップ

```bash
# 1. リポジトリクローン
git clone https://github.com/quantumshiro/polytorus.git
cd polytorus

# 2. ビルド
cargo build --release

# 3. テスト実行
cargo test --lib
./scripts/quality_check.sh
```

### Step 2: ネットワーク設定

```bash
# 1. 設定ファイル作成
mkdir -p config/testnet

# 2. ノード設定（config/testnet/node1.toml）
cat > config/testnet/node1.toml << EOF
[network]
listen_addr = "0.0.0.0:8001"
bootstrap_peers = []
max_peers = 50

[consensus]
block_time = 10000
difficulty = 4
max_block_size = 1048576

[execution]
gas_limit = 8000000
gas_price = 1

[settlement]
challenge_period = 100
batch_size = 100
min_validator_stake = 1000

[data_availability]
retention_period = 604800
max_data_size = 1048576
EOF

# 3. 追加ノード設定（ポート番号を変更）
cp config/testnet/node1.toml config/testnet/node2.toml
sed -i 's/8001/8002/g' config/testnet/node2.toml

cp config/testnet/node1.toml config/testnet/node3.toml  
sed -i 's/8001/8003/g' config/testnet/node3.toml

cp config/testnet/node1.toml config/testnet/node4.toml
sed -i 's/8001/8004/g' config/testnet/node4.toml
```

### Step 3: ノード起動

```bash
# 1. ノード1（ブートストラップノード）
./target/release/polytorus \
  --config config/testnet/node1.toml \
  --data-dir data/testnet/node1 \
  --http-port 9001 \
  --modular-start &

# 2. ノード2-4（順次起動）
./target/release/polytorus \
  --config config/testnet/node2.toml \
  --data-dir data/testnet/node2 \
  --http-port 9002 \
  --modular-start &

./target/release/polytorus \
  --config config/testnet/node3.toml \
  --data-dir data/testnet/node3 \
  --http-port 9003 \
  --modular-start &

./target/release/polytorus \
  --config config/testnet/node4.toml \
  --data-dir data/testnet/node4 \
  --http-port 9004 \
  --modular-start &

# 3. ネットワーク接続確認
sleep 10
curl http://localhost:9001/api/health
curl http://localhost:9001/api/network/status
```

### Step 4: ネットワーク動作確認

```bash
# 1. ウォレット作成
./target/release/polytorus --createwallet --data-dir data/testnet/node1

# 2. アドレス確認
./target/release/polytorus --listaddresses --data-dir data/testnet/node1

# 3. ERC20トークン展開テスト
./target/release/polytorus \
  --smart-contract-deploy erc20 \
  --data-dir data/testnet/node1 \
  --http-port 9001

# 4. トランザクション送信テスト
curl -X POST http://localhost:9001/api/transaction \
  -H "Content-Type: application/json" \
  -d '{"type":"transfer","amount":100,"recipient":"target_address"}'

# 5. ネットワーク同期確認
./target/release/polytorus --network-sync --data-dir data/testnet/node2
```

### Step 5: 監視とログ

```bash
# 1. ネットワーク統計
curl http://localhost:9001/api/stats
curl http://localhost:9001/api/network/peers

# 2. ログ監視
tail -f data/testnet/node1/logs/polytorus.log

# 3. リアルタイム統計（別ターミナル）
cargo run --example transaction_monitor
```

## 🔒 パブリックテストネットに向けた追加実装

### 重要な実装ギャップ

#### 1. Genesis Block Management

**現在の状況:** 手動での初期化のみ
**必要な実装:**
```rust
// src/genesis/mod.rs (新規作成必要)
pub struct GenesisConfig {
    pub chain_id: u64,
    pub initial_validators: Vec<ValidatorInfo>,
    pub initial_balances: HashMap<String, u64>,
    pub consensus_params: ConsensusParams,
}

impl GenesisConfig {
    pub fn generate_genesis_block(&self) -> Result<Block> {
        // Genesis block生成ロジック
    }
}
```

#### 2. Validator Set Management

**現在の状況:** 基本的なバリデーター情報のみ
**必要な実装:**
```rust
// src/staking/mod.rs (新規作成必要)
pub struct StakingManager {
    pub fn stake(&mut self, validator: Address, amount: u64) -> Result<()>;
    pub fn unstake(&mut self, validator: Address, amount: u64) -> Result<()>;
    pub fn slash(&mut self, validator: Address, reason: SlashReason) -> Result<()>;
    pub fn get_active_validators(&self) -> Vec<ValidatorInfo>;
}
```

#### 3. Network Bootstrap

**現在の状況:** 静的ピア設定
**必要な実装:**
```rust
// src/network/bootstrap.rs (拡張必要)
pub struct BootstrapManager {
    pub async fn discover_peers(&self) -> Result<Vec<PeerInfo>>;
    pub async fn register_node(&self, node_info: NodeInfo) -> Result<()>;
    pub fn get_bootstrap_nodes(&self) -> Vec<BootstrapNode>;
}
```

#### 4. Security Hardening

**必要な追加実装:**
- TLS/SSL証明書管理
- API認証システム
- DDoS防護機構
- ファイアウォール設定

### 実装優先度

| 優先度 | 機能 | 実装工数 | 影響範囲 |
|--------|------|---------|---------|
| **HIGH** | Genesis Block Generator | 2-3日 | 全体 |
| **HIGH** | TLS/SSL Infrastructure | 1-2日 | セキュリティ |
| **MEDIUM** | Validator Staking | 3-5日 | コンセンサス |
| **MEDIUM** | Bootstrap Discovery | 2-3日 | ネットワーク |
| **LOW** | Auto-scaling | 5-7日 | 運用 |

## 🧪 テストシナリオ

### 基本機能テスト

```bash
# 1. ノード起動テスト
./scripts/test_node_startup.sh

# 2. P2P接続テスト  
./scripts/test_p2p_connectivity.sh

# 3. トランザクション伝播テスト
./scripts/test_complete_propagation.sh

# 4. スマートコントラクトテスト
cargo test erc20_integration_tests

# 5. パフォーマンステスト
./scripts/benchmark_tps.sh
```

### 負荷テスト

```bash
# 1. 高負荷トランザクション
cargo run --example stress_test -- --duration 300 --tps 100

# 2. 大量ノードテスト
./scripts/simulate.sh local --nodes 20 --duration 600

# 3. ネットワーク分断テスト
./scripts/test_network_partition.sh
```

## 🚨 トラブルシューティング

### よくある問題

#### 1. OpenFHE依存関係エラー
```bash
# 解決方法
export OPENFHE_ROOT=/usr/local
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
sudo ldconfig
```

#### 2. ポート競合
```bash
# 使用中ポート確認
netstat -tuln | grep :900

# プロセス終了
pkill -f polytorus
```

#### 3. ストレージ容量不足
```bash
# ログファイル削除
find data/ -name "*.log" -mtime +7 -delete

# 古いブロックデータ削除
rm -rf data/*/blockchain/blocks/00*
```

#### 4. ネットワーク同期問題
```bash
# 強制再同期
./target/release/polytorus --network-sync --data-dir data/node1

# ピア接続リセット
./target/release/polytorus --network-reset --data-dir data/node1
```

### ログ分析

```bash
# エラーログ抽出
grep "ERROR" data/testnet/node1/logs/polytorus.log

# パフォーマンス統計
grep "TPS\|latency" data/testnet/node1/logs/polytorus.log

# ネットワーク統計
curl http://localhost:9001/api/network/stats | jq .
```

## 📊 現在のテストネット展開可能性

### ✅ 即座に可能（今日から）

- **ローカル開発ネットワーク**: 1-10ノード
- **プライベートコンソーシアム**: 既知の参加者による内部テスト
- **概念実証**: Diamond IO、モジュラーアーキテクチャのデモ

### 🔧 1-2週間で可能

- **セミプライベートテストネット**: 追加セキュリティ実装後
- **外部開発者向けテスト**: API公開とドキュメント整備後

### 🎯 1-2ヶ月で可能

- **パブリックテストネット**: 完全なGenesis管理とセキュリティ実装後
- **本格的なバリデーターネットワーク**: ステーキング機能実装後

## 🎉 結論

PolyTorusは**現在でも高品質なプライベートテストネット**の展開が可能であり、**75%の完成度**を達成しています。モジュラーアーキテクチャの革新性と実装品質は非常に高く、追加の実装により完全なパブリックテストネットの展開も実現可能です。

**推奨されるアプローチ:**
1. **Phase 1 (即座)**: プライベートコンソーシアムテストネット
2. **Phase 2 (2-4週間)**: セミプライベートテストネット  
3. **Phase 3 (1-2ヶ月)**: パブリックテストネット

この段階的アプローチにより、リスクを最小化しながら確実にテストネットを公開できます。