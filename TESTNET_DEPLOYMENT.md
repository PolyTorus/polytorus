# PolyTorus テストネット展開ガイド

このドキュメントでは、PolyTorusブロックチェーンのテストネットを様々な環境で展開する方法を説明します。

## 目次

1. [ローカルテストネット](#ローカルテストネット)
2. [EC2分散テストネット](#ec2分散テストネット)
3. [Dockerクラスター展開](#dockerクラスター展開)
4. [マイニング設定](#マイニング設定)
5. [ネットワーク監視とメンテナンス](#ネットワーク監視とメンテナンス)
6. [トラブルシューティング](#トラブルシューティング)

## ローカルテストネット

### 前提条件

- Rust 1.87 nightly以降
- OpenFHE (MachinaIO fork)
- システム依存関係: `cmake`, `libgmp-dev`, `libntl-dev`, `libboost-all-dev`

### 1. 環境セットアップ

```bash
# プロジェクトビルド
cargo build --release

# テストネット設定ディレクトリ作成
mkdir -p testnet-config

# データディレクトリ作成
mkdir -p testnet-data testnet-data-2
```

### 2. ノード1設定 (testnet-config/testnet.toml)

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

[mempool]
max_transactions = 10000
max_transaction_age = "3600s"
min_fee = 1

[rpc]
enabled = true
bind_address = "127.0.0.1:8545"
max_connections = 100
```

### 3. ノード2設定 (testnet-config/testnet-node2.toml)

```toml
[network]
chain_id = "polytorus-testnet-1"
network_name = "PolyTorus Testnet"
p2p_port = 8001
rpc_port = 8546
discovery_port = 8901
max_peers = 50

[consensus]
block_time = 6000
difficulty = 2
max_block_size = 1048576

[diamond_io]
mode = "Testing"
ring_dimension = 1024
noise_bound = 6.4

[storage]
data_dir = "./testnet-data-2"
cache_size = 1000

[bootstrap]
nodes = [
    "127.0.0.1:8000"  # 最初のノードをブートストラップとして使用
]

[mempool]
max_transactions = 10000
max_transaction_age = "3600s"
min_fee = 1

[rpc]
enabled = true
bind_address = "127.0.0.1:8546"
max_connections = 100
```

### 4. ノード起動

```bash
# ノード1初期化と起動
./target/release/polytorus --modular-init --data-dir ./testnet-data --config testnet-config/testnet.toml
./target/release/polytorus --modular-start --data-dir ./testnet-data --config testnet-config/testnet.toml --http-port 8080 > testnet.log 2>&1 &

# ノード2初期化と起動
./target/release/polytorus --modular-init --data-dir ./testnet-data-2 --config testnet-config/testnet-node2.toml
./target/release/polytorus --modular-start --data-dir ./testnet-data-2 --config testnet-config/testnet-node2.toml --http-port 8081 > testnet-node2.log 2>&1 &
```

### 5. 動作確認

```bash
# ヘルスチェック
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8081/health

# ノード状態確認
curl http://127.0.0.1:8080/status
curl http://127.0.0.1:8081/status

# トランザクション送信テスト
curl -X POST http://127.0.0.1:8080/transaction \
  -H "Content-Type: application/json" \
  -d '{"from":"test-addr-1","to":"test-addr-2","amount":100}'

# 統計情報確認
curl http://127.0.0.1:8080/stats
curl http://127.0.0.1:8081/stats
```

## EC2分散テストネット

### アーキテクチャ概要

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   EC2 Node 1    │────│   EC2 Node 2    │────│   EC2 Node 3    │
│  us-east-1      │    │  eu-west-1      │    │  ap-southeast-1 │
│  P2P: 8000      │    │  P2P: 8000      │    │  P2P: 8000      │
│  API: 8080      │    │  API: 8080      │    │  API: 8080      │
│  RPC: 8545      │    │  RPC: 8545      │    │  RPC: 8545      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 1. EC2インスタンス作成

**推奨スペック:**
- インスタンスタイプ: `t3.medium` 以上 (2 vCPU, 4GB RAM)
- OS: Ubuntu 22.04 LTS
- ストレージ: 20GB gp3
- セキュリティグループ: 以下のポート開放
  - SSH (22)
  - P2P (8000)
  - HTTP API (8080) 
  - RPC (8545)
  - Discovery (8900)

### 2. 自動セットアップスクリプト実行

各EC2インスタンスで以下を実行:

```bash
# リポジトリクローン
git clone https://github.com/PolyTorus/polytorus.git
cd polytorus

# 自動セットアップ実行
chmod +x deployment/ec2-setup.sh
./deployment/ec2-setup.sh
```

### 3. ネットワーク設定の更新

最初のノード起動後、各ノードの設定ファイル `~/polytorus-testnet.toml` を編集:

```toml
[bootstrap]
nodes = [
    "FIRST_NODE_PUBLIC_IP:8000",
    "SECOND_NODE_PUBLIC_IP:8000"
]
```

### 4. ノード起動と管理

```bash
# ノード起動
sudo systemctl start polytorus

# 状態確認
sudo systemctl status polytorus

# ログ確認
sudo journalctl -u polytorus -f

# 設定リロード
sudo systemctl restart polytorus
```

### 5. グローバルネットワーク確認

```bash
# 各ノードの外部アクセステスト
curl http://FIRST_NODE_IP:8080/status
curl http://SECOND_NODE_IP:8080/status
curl http://THIRD_NODE_IP:8080/status

# P2P接続確認
curl http://FIRST_NODE_IP:8080/network/peers
```

## Dockerクラスター展開

### Docker Compose使用

```bash
# 分散Docker環境起動
cd docker
docker-compose -f docker-compose.distributed.yml up -d

# ログ確認
docker-compose -f docker-compose.distributed.yml logs -f

# スケール拡張
docker-compose -f docker-compose.distributed.yml up -d --scale polytorus-node-2=3
```

### Kubernetes展開 (オプション)

```yaml
# k8s/polytorus-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: polytorus-testnet
spec:
  replicas: 3
  selector:
    matchLabels:
      app: polytorus
  template:
    metadata:
      labels:
        app: polytorus
    spec:
      containers:
      - name: polytorus
        image: polytorus:distributed
        ports:
        - containerPort: 8000
        - containerPort: 8080
        - containerPort: 8545
        env:
        - name: RUST_LOG
          value: "info"
---
apiVersion: v1
kind: Service
metadata:
  name: polytorus-service
spec:
  selector:
    app: polytorus
  ports:
  - name: p2p
    port: 8000
    targetPort: 8000
  - name: api
    port: 8080
    targetPort: 8080
  - name: rpc
    port: 8545
    targetPort: 8545
  type: LoadBalancer
```

## マイニング設定

### 1. マイニング用ウォレット作成

```bash
# ウォレット作成
./target/release/polytorus --createwallet --data-dir ./testnet-data

# アドレス一覧表示
./target/release/polytorus --listaddresses --data-dir ./testnet-data
```

### 2. マイニング開始

```bash
# コンセンサス層でのマイニング
# PolyTorusは統合されたmodular architectureでマイニングを実行
# consensus.rs の mine_block() 関数が自動的に呼び出されます

# マイニング統計確認
curl http://localhost:8080/stats
```

### 3. マイニング設定調整

```toml
[consensus]
block_time = 6000      # ブロック時間 (ミリ秒)
difficulty = 2         # 難易度 (1-32)
max_block_size = 1048576  # 最大ブロックサイズ
```

### 4. マイニングプール設定 (将来対応)

```toml
[mining_pool]
enabled = false
pool_address = "pool.polytorus.network:8333"
worker_name = "worker1"
```

## ネットワーク監視とメンテナンス

### 監視ダッシュボード

```bash
# ネットワーク状態監視
watch -n 5 'curl -s http://localhost:8080/status | jq'

# トランザクション処理監視
watch -n 2 'curl -s http://localhost:8080/stats | jq'

# P2P接続監視
curl http://localhost:8080/network/health
```

### ログ分析

```bash
# エラーログ抽出
sudo journalctl -u polytorus | grep ERROR

# P2P接続ログ
sudo journalctl -u polytorus | grep "peer\|P2P"

# マイニングログ
sudo journalctl -u polytorus | grep "mine\|block"
```

### パフォーマンスチューニング

```toml
[performance]
# メモリプール設定
max_transactions = 20000
cache_size = 2000

# ネットワーク設定
max_peers = 100
connection_timeout = 30000

# 同期設定
sync_batch_size = 1000
sync_timeout = 60000
```

## トラブルシューティング

### よくある問題と解決方法

#### 1. ノード間接続失敗

```bash
# ファイアウォール確認
sudo ufw status

# ポート開放
sudo ufw allow 8000/tcp
sudo ufw allow 8080/tcp
sudo ufw allow 8545/tcp

# ネットワーク接続テスト
telnet OTHER_NODE_IP 8000
```

#### 2. OpenFHE依存関係エラー

```bash
# OpenFHE再インストール
sudo rm -rf /usr/local/include/openfhe
sudo ./scripts/install_openfhe.sh

# 環境変数設定
export OPENFHE_ROOT=/usr/local
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
```

#### 3. データベースロックエラー

```bash
# プロセス確認と停止
ps aux | grep polytorus
kill -9 PID

# データディレクトリクリーンアップ
rm -rf ./testnet-data/modular_storage/*.lock
```

#### 4. メモリ不足

```bash
# システムリソース確認
free -h
df -h

# スワップ追加
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

### ログレベル調整

```bash
# デバッグモードで起動
RUST_LOG=debug ./target/release/polytorus --modular-start

# 特定モジュールのみ詳細ログ
RUST_LOG=polytorus::modular::consensus=debug ./target/release/polytorus --modular-start
```

### ネットワーク診断ツール

```bash
# P2P接続状態
curl http://localhost:8080/network/peers | jq

# ネットワークトポロジー
curl http://localhost:8080/network/topology | jq

# メッセージキュー統計
curl http://localhost:8080/network/queue-stats | jq
```

## 高度な設定

### セキュリティ強化

```toml
[security]
enable_rate_limiting = true
max_requests_per_minute = 1000
allowed_origins = ["https://app.polytorus.network"]
api_key_required = true
```

### 暗号化設定

```toml
[diamond_io]
mode = "Production"  # 本番環境用高セキュリティ
ring_dimension = 2048
noise_bound = 3.2
encryption_level = "Maximum"
```

### 負荷分散設定

```toml
[load_balancing]
enable_auto_scaling = true
min_nodes = 3
max_nodes = 10
cpu_threshold = 80
memory_threshold = 85
```

## 検証とテスト

### 機能テストスイート

```bash
# 完全なテストスイート実行
cargo test --lib

# P2Pネットワークテスト
cargo test network_tests --nocapture

# コンセンサステスト
cargo test consensus_tests --nocapture

# Diamond IOテスト
cargo test diamond_io_tests --nocapture
```

### パフォーマンステスト

```bash
# ベンチマークテスト
cargo bench

# トランザクション処理性能テスト
./scripts/test_complete_propagation.sh

# マルチノードシミュレーション
./scripts/simulate.sh local --nodes 4 --duration 300
```

### セキュリティ監査

```bash
# Kani形式検証
make kani-verify

# セキュリティ監査
cargo audit

# 依存関係チェック
cargo outdated
```

## サポートとコミュニティ

- **ドキュメント**: [docs.polytorus.network](https://docs.polytorus.network)
- **GitHub**: [github.com/PolyTorus/polytorus](https://github.com/PolyTorus/polytorus)
- **Discord**: [discord.gg/polytorus](https://discord.gg/polytorus)
- **テストネットエクスプローラー**: [testnet.polytorus.network](https://testnet.polytorus.network)

このガイドにより、ローカル環境から本格的なグローバル分散テストネットまで、様々なスケールでPolyTorusブロックチェーンを展開できます。