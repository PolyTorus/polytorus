# モジューラーブロックチェーン CLI コマンド

## 概要
PolyTorusのモジューラーアーキテクチャを操作するためのCLIコマンドを提供します。

## 新規コマンド

### `modular`
モジューラーブロックチェーンの管理コマンド

```bash
# モジューラーノードの開始
polytorus modular start --config config/modular.toml

# レイヤー状態の確認
polytorus modular status

# 実行レイヤーの状態表示
polytorus modular execution status

# セトルメントレイヤーの状態表示
polytorus modular settlement status

# コンセンサスレイヤーの状態表示
polytorus modular consensus status

# データ可用性レイヤーの状態表示
polytorus modular data-availability status
```

### `layers`
レイヤー固有の操作コマンド

```bash
# 実行レイヤーでトランザクション実行
polytorus layers execution execute-tx --tx-file transaction.json

# セトルメントのバッチ送信
polytorus layers settlement submit-batch --batch-file batch.json

# 不正証明の送信
polytorus layers settlement submit-challenge --challenge-file challenge.json

# データの保存
polytorus layers data-availability store --data-file data.bin

# データの取得
polytorus layers data-availability retrieve --hash <HASH>
```

### `config`
設定管理コマンド

```bash
# モジューラー設定の生成
polytorus config generate-modular --output config/modular.toml

# 設定の検証
polytorus config validate --config config/modular.toml

# レイヤー固有設定の表示
polytorus config show-layer --layer execution
polytorus config show-layer --layer consensus
polytorus config show-layer --layer settlement
polytorus config show-layer --layer data-availability
```

## 設定ファイル例

### `config/modular.toml`
```toml
[execution]
gas_limit = 8000000
gas_price = 1

[execution.wasm_config]
max_memory_pages = 256
max_stack_size = 65536
gas_metering = true

[settlement]
challenge_period = 100
batch_size = 100
min_validator_stake = 1000

[consensus]
block_time = 10000
difficulty = 4
max_block_size = 1048576

[data_availability]
retention_period = 604800
max_data_size = 1048576

[data_availability.network_config]
listen_addr = "0.0.0.0:7000"
bootstrap_peers = []
max_peers = 50
```

## 使用例

### 1. モジューラーノードの起動
```bash
# 設定ファイルの生成
polytorus config generate-modular --output config/modular.toml

# ノードの起動
polytorus modular start --config config/modular.toml
```

### 2. トランザクションの実行
```bash
# トランザクションファイルの作成
cat > transaction.json << EOF
{
  "to": "recipient_address",
  "value": 100,
  "gas_limit": 21000
}
EOF

# トランザクションの実行
polytorus layers execution execute-tx --tx-file transaction.json
```

### 3. レイヤー状態の監視
```bash
# 全体状態の確認
polytorus modular status

# 実行レイヤーの詳細確認
polytorus layers execution status

# セトルメントの履歴確認
polytorus layers settlement history --limit 10
```

### 4. データの保存と取得
```bash
# データの保存
echo "Hello, Modular Blockchain!" > data.txt
polytorus layers data-availability store --data-file data.txt

# データの取得（上記コマンドで返されたハッシュを使用）
polytorus layers data-availability retrieve --hash abc123...
```

## エラーハンドリング

### 一般的なエラー
- `Layer not responding`: レイヤーが応答していない
- `Invalid configuration`: 設定ファイルが無効
- `Gas limit exceeded`: ガス制限を超過
- `Challenge period expired`: チャレンジ期間が終了

### デバッグオプション
```bash
# 詳細ログ出力
RUST_LOG=debug polytorus modular start --config config/modular.toml

# 特定レイヤーのログ
RUST_LOG=polytorus::modular::execution=trace polytorus modular start
```

## パフォーマンス監視

### メトリクス確認
```bash
# レイヤー別パフォーマンス
polytorus modular metrics --layer execution
polytorus modular metrics --layer consensus
polytorus modular metrics --layer settlement
polytorus modular metrics --layer data-availability

# 全体的な統計
polytorus modular statistics
```

## 開発者向け機能

### テスト環境の設定
```bash
# テスト用設定の生成
polytorus config generate-modular --test --output config/test-modular.toml

# テスト用データの初期化
polytorus modular init-test --config config/test-modular.toml
```

### プロファイリング
```bash
# パフォーマンスプロファイリングの有効化
polytorus modular start --config config/modular.toml --profile

# メモリ使用量の監視
polytorus modular memory-usage --interval 5s
```
