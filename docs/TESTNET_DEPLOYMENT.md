# PolyTorus Testnet Deployment Guide

このドキュメントは、PolyTorus ブロックチェーンのテストネットを展開し、運用するための完全なガイドです。

## 概要

PolyTorus は次世代のモジュラーブロックチェーンプラットフォームで、ポスト量子暗号化、Diamond IO統合、および革新的なモジュラーアーキテクチャを特徴としています。

### 主要機能
- **モジュラーアーキテクチャ**: 実行、決済、合意、データ可用性の分離されたレイヤー
- **Diamond IO プライバシー**: 区別不可能難読化による高度なプライバシー保護
- **ポスト量子暗号**: FN-DSA署名による量子耐性
- **VerkleTree**: 効率的な状態コミットメント
- **P2P ネットワーキング**: DHT様のピア発見とメッセージ優先順位付け
- **包括的RPC API**: Ethereum互換エンドポイント

## システム要件

### 最小要件
- **OS**: Linux (Ubuntu 20.04+ 推奨)
- **CPU**: 4コア以上
- **RAM**: 8GB以上
- **Storage**: 100GB以上 SSD
- **Network**: 1Mbps以上の安定したインターネット接続

### 推奨要件
- **OS**: Linux (Ubuntu 22.04 LTS)
- **CPU**: 8コア以上
- **RAM**: 16GB以上
- **Storage**: 500GB以上 NVMe SSD
- **Network**: 10Mbps以上の安定したインターネット接続

## 前提条件

### 1. Rust インストール
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default nightly
```

### 2. 必要なシステムライブラリ
```bash
sudo apt update
sudo apt install -y cmake libgmp-dev libntl-dev libboost-all-dev \
    build-essential pkg-config libssl-dev git curl
```

### 3. OpenFHE インストール
```bash
# 自動インストールスクリプトを実行
sudo ./scripts/install_openfhe.sh

# 環境変数を設定
export OPENFHE_ROOT=/usr/local
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH

# .bashrc に永続化
echo 'export OPENFHE_ROOT=/usr/local' >> ~/.bashrc
echo 'export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH' >> ~/.bashrc
echo 'export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH' >> ~/.bashrc
```

## ビルドとテスト

### 1. プロジェクトのクローンとビルド
```bash
git clone https://github.com/PolyTorus/polytorus.git
cd polytorus
git checkout feature/testnet

# 依存関係のビルドとテスト
cargo build --release
cargo test --lib
```

### 2. コード品質チェック
```bash
# 包括的な品質チェック
make pre-commit

# または個別実行
cargo fmt
cargo clippy --all-targets --all-features -- -W clippy::all
cargo test
```

### 3. Diamond IO テスト
```bash
# Diamond IO 統合テスト
cargo test diamond_io --nocapture

# パフォーマンステスト
cargo run --example diamond_io_performance_test
```

## ノード設定

### 1. 設定ファイルの作成

#### テストネット設定 (`config/testnet.toml`)
```toml
[network]
chain_id = "polytorus-testnet-1"
network_name = "PolyTorus Testnet"
p2p_port = 8000
rpc_port = 8545
discovery_port = 8900
max_peers = 50

[consensus]
block_time = 6000  # 6秒
difficulty = 2     # テストネット用低難易度
max_block_size = 1048576  # 1MB

[diamond_io]
mode = "Testing"
ring_dimension = 1024
noise_bound = 6.4

[storage]
data_dir = "./testnet-data"
cache_size = 1000

[bootstrap]
nodes = [
    "testnet-seed1.polytorus.io:8000",
    "testnet-seed2.polytorus.io:8000",
    "testnet-seed3.polytorus.io:8000"
]
```

#### バリデータ設定 (`config/validator.toml`)
```toml
[validator]
enabled = true
address = "polytorus1validator1qqqqqqqqqqqqqqqqqqqqqqqqqqq8yf5ce"
stake = 100000000  # 100M tokens
commission_rate = 0.05  # 5%

[mining]
enabled = true
threads = 4
target_gas_limit = 8000000
```

### 2. ジェネシスブロック設定

#### デフォルトテストネットジェネシス
```bash
# デフォルトのテストネットジェネシスを使用
./target/release/polytorus modular genesis --config config/testnet.toml --export genesis.json
```

#### カスタムジェネシス (`genesis-custom.json`)
```json
{
  "chain_id": "polytorus-testnet-1",
  "network_name": "PolyTorus Testnet",
  "timestamp": 0,
  "difficulty": 2,
  "gas_limit": 8000000,
  "allocations": {
    "polytorus1test1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqq8yf5ce": {
      "balance": 1000000000000000,
      "nonce": 0,
      "code": null,
      "storage": {}
    }
  },
  "validators": [
    {
      "address": "polytorus1validator1qqqqqqqqqqqqqqqqqqqqqqqqqqq8yf5ce",
      "stake": 100000000,
      "public_key": "validator_pubkey_here",
      "commission_rate": 0.05
    }
  ]
}
```

## ノードの起動

### 1. フルノードの起動
```bash
# バックグラウンドで実行
nohup ./target/release/polytorus modular start \
    --config config/testnet.toml \
    --genesis genesis.json \
    --data-dir ./testnet-data \
    > node.log 2>&1 &

# ログの確認
tail -f node.log
```

### 2. バリデータノードの起動
```bash
# バリデータモードで起動
nohup ./target/release/polytorus modular start \
    --config config/testnet.toml \
    --validator-config config/validator.toml \
    --genesis genesis.json \
    --data-dir ./validator-data \
    --enable-mining \
    > validator.log 2>&1 &
```

### 3. ライトノードの起動
```bash
# ライトノードモード
./target/release/polytorus modular start \
    --config config/testnet.toml \
    --light-mode \
    --data-dir ./light-data
```

## ウォレット操作

### 1. ウォレットの作成
```bash
# ポスト量子署名ウォレット
./target/release/polytorus createwallet FNDSA

# 従来のECDSAウォレット
./target/release/polytorus createwallet ECDSA

# ウォレット一覧表示
./target/release/polytorus listaddresses
```

### 2. 残高確認とトランザクション
```bash
# 残高確認
./target/release/polytorus getbalance <address>

# トランザクション送信
./target/release/polytorus send \
    --from <from_address> \
    --to <to_address> \
    --amount 1000000 \
    --fee 1000
```

## マイニング

### 1. ソロマイニング
```bash
# 指定アドレスでマイニング開始
./target/release/polytorus modular mine <miner_address>

# マイニング統計確認
./target/release/polytorus modular stats
```

### 2. プールマイニング
```bash
# マイニングプール参加
./target/release/polytorus modular mine \
    --pool-address <pool_address> \
    --worker-name <worker_name>
```

## モニタリング

### 1. ノード状態確認
```bash
# 基本情報
./target/release/polytorus modular state

# レイヤー情報
./target/release/polytorus modular layers

# ネットワーク情報
./target/release/polytorus modular network
```

### 2. RPC API 使用
```bash
# チェーン情報取得
curl -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
    http://localhost:8545

# 最新ブロック番号取得
curl -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    http://localhost:8545

# 残高確認
curl -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_getBalance","params":["<address>","latest"],"id":1}' \
    http://localhost:8545
```

### 3. メトリクス監視
```bash
# Prometheusメトリクス (HTTPサーバーが有効な場合)
curl http://localhost:8080/metrics

# ノード健全性チェック
curl http://localhost:8080/health
```

## 複数ノードシミュレーション

### 1. ローカルテストネット
```bash
# 4ノードシミュレーション
./scripts/simulate.sh local --nodes 4 --duration 300

# トランザクション伝播テスト
./scripts/test_complete_propagation.sh
```

### 2. ネットワーク接続テスト
```bash
# トランザクション監視
cargo run --example transaction_monitor

# ネットワーク健全性チェック
./target/release/polytorus modular network --check-health
```

## トラブルシューティング

### 1. 一般的な問題

#### OpenFHE依存関係エラー
```bash
# OpenFHEライブラリの確認
ls -la /usr/local/lib/libopenfhe*

# 環境変数の確認
echo $OPENFHE_ROOT
echo $LD_LIBRARY_PATH
```

#### P2Pネットワーク接続問題
```bash
# ファイアウォール設定確認
sudo ufw status

# ポート開放
sudo ufw allow 8000/tcp
sudo ufw allow 8900/udp

# ネットワーク接続テスト
telnet <peer_ip> 8000
```

#### データベース破損
```bash
# データディレクトリのクリーンアップ
rm -rf ./testnet-data
mkdir ./testnet-data

# ジェネシスから再同期
./target/release/polytorus modular start --reset-data
```

### 2. ログ分析
```bash
# エラーログの確認
grep -i error node.log
grep -i warn node.log

# パフォーマンス監視
grep "Block mined" node.log | tail -10
grep "Sync progress" node.log | tail -10
```

### 3. デバッグモード
```bash
# デバッグレベルのログ出力
RUST_LOG=debug ./target/release/polytorus modular start

# トレースレベル（詳細）
RUST_LOG=trace ./target/release/polytorus modular start
```

## セキュリティ考慮事項

### 1. ノードセキュリティ
- ウォレットの秘密鍵を安全に保管
- ファイアウォールで不要なポートを閉鎖
- 定期的なシステムアップデート
- SSL/TLS証明書の使用（本番環境）

### 2. ネットワークセキュリティ
- VPNの使用を推奨
- DDoS保護の実装
- レート制限の設定
- 信頼できるピアとの接続

### 3. 運用セキュリティ
```bash
# ファイル権限の設定
chmod 600 config/*.toml
chmod 700 testnet-data/

# バックアップの作成
tar -czf backup-$(date +%Y%m%d).tar.gz testnet-data/ config/
```

## パフォーマンス最適化

### 1. システム最適化
```bash
# ファイルディスクリプタ制限の増加
echo '* soft nofile 65536' >> /etc/security/limits.conf
echo '* hard nofile 65536' >> /etc/security/limits.conf

# TCP設定の最適化
echo 'net.core.rmem_max = 16777216' >> /etc/sysctl.conf
echo 'net.core.wmem_max = 16777216' >> /etc/sysctl.conf
sysctl -p
```

### 2. アプリケーション最適化
```bash
# 並列処理スレッド数の調整
export RAYON_NUM_THREADS=8

# メモリプール設定
export POLYTORUS_MEMPOOL_SIZE=10000
export POLYTORUS_CACHE_SIZE=2000
```

## API リファレンス

### JSON-RPC エンドポイント

#### Ethereum互換API
- `eth_chainId` - チェーンID取得
- `eth_blockNumber` - 最新ブロック番号
- `eth_getBalance` - アカウント残高
- `eth_sendTransaction` - トランザクション送信
- `eth_getTransactionReceipt` - トランザクション受信

#### PolyTorus固有API
- `polytorus_getModularState` - モジュラー状態
- `polytorus_getDiamondIOStats` - Diamond IO統計
- `polytorus_getValidatorInfo` - バリデータ情報
- `polytorus_getNetworkTopology` - ネットワークトポロジー

### WebSocket API
```javascript
// WebSocket接続例
const ws = new WebSocket('ws://localhost:8546');
ws.send(JSON.stringify({
    jsonrpc: '2.0',
    method: 'eth_subscribe',
    params: ['newHeads'],
    id: 1
}));
```

## 本番環境への移行

### 1. メインネット設定の変更
```toml
[network]
chain_id = "polytorus-mainnet-1"
network_name = "PolyTorus Mainnet"
difficulty = 6  # 高難易度

[diamond_io]
mode = "Production"  # 本番セキュリティ
ring_dimension = 2048
```

### 2. セキュリティ強化
- HSM（Hardware Security Module）の使用
- マルチシグウォレットの実装
- 監査ログの設定
- 侵入検知システムの導入

### 3. スケーリング対策
- ロードバランサーの設定
- レプリケーションの実装
- CDNの利用
- 自動スケーリング

## サポートとコミュニティ

### 公式リソース
- **GitHub**: https://github.com/PolyTorus/polytorus
- **Discord**: https://discord.gg/polytorus
- **Telegram**: https://t.me/polytorusofficial
- **Twitter**: https://twitter.com/PolyTorusChain

### 技術サポート
- **Issue報告**: GitHub Issues
- **技術質問**: Discord #development チャンネル
- **緊急時**: support@polytorus.io

### 貢献方法
1. Forkしてfeatureブランチを作成
2. 変更を実装しテストを追加
3. `make pre-commit`でコード品質を確認
4. Pull Requestを送信

---

このガイドは PolyTorus v0.1.0 に基づいています。最新情報は公式ドキュメントを確認してください。