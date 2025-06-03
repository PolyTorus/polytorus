# PolyTorus モジューラーブロックチェーンアーキテクチャ

## 概要
PolyTorusをモジューラーブロックチェーンとして設計し、各レイヤーを独立して開発・運用できるアーキテクチャを構築します。

## アーキテクチャレイヤー

### 1. 実行レイヤー (Execution Layer)
- **役割**: トランザクション実行とスマートコントラクト処理
- **責任範囲**: 
  - 状態遷移ロジック
  - WASM実行環境
  - ガス計測とリソース管理
- **独立性**: 他のレイヤーから分離され、プラガブル

### 2. セトルメントレイヤー (Settlement Layer)
- **役割**: 最終的な状態確定と紛争解決
- **責任範囲**:
  - トランザクションの最終確定
  - 不正証明の検証
  - ルート状態の管理
- **独立性**: コンセンサスとデータ可用性から分離

### 3. コンセンサスレイヤー (Consensus Layer)
- **役割**: ブロック順序決定とバリデーター管理
- **責任範囲**:
  - プルーフ・オブ・ワーク
  - バリデーター選択
  - フォーク解決
- **独立性**: 実行とデータ可用性から分離

### 4. データ可用性レイヤー (Data Availability Layer)
- **役割**: データの保存と配布
- **責任範囲**:
  - ブロックデータの保存
  - P2Pネットワーク通信
  - データ同期
- **独立性**: 実行とコンセンサスから分離

## モジュール間通信インターフェース

### レイヤー間API
```rust
// 実行レイヤーインターフェース
pub trait ExecutionLayer {
    fn execute_block(&self, block: Block) -> Result<ExecutionResult>;
    fn get_state_root(&self) -> Hash;
    fn verify_execution(&self, proof: ExecutionProof) -> bool;
}

// セトルメントレイヤーインターフェース
pub trait SettlementLayer {
    fn settle_batch(&self, batch: ExecutionBatch) -> Result<SettlementResult>;
    fn verify_fraud_proof(&self, proof: FraudProof) -> bool;
    fn get_settlement_root(&self) -> Hash;
}

// コンセンサスレイヤーインターフェース
pub trait ConsensusLayer {
    fn propose_block(&self, block: Block) -> Result<()>;
    fn validate_block(&self, block: Block) -> bool;
    fn get_canonical_chain(&self) -> Vec<Hash>;
}

// データ可用性レイヤーインターフェース
pub trait DataAvailabilityLayer {
    fn store_data(&self, data: &[u8]) -> Result<Hash>;
    fn retrieve_data(&self, hash: Hash) -> Result<Vec<u8>>;
    fn verify_availability(&self, hash: Hash) -> bool;
}
```

## 実装方針

### Phase 1: 現在のモノリシック構造の分析と分離
1. 既存コードの依存関係マッピング
2. レイヤー境界の明確化
3. インターフェース定義

### Phase 2: インターフェース実装
1. トレイト定義とモック実装
2. レイヤー間通信プロトコル
3. 設定とランタイム管理

### Phase 3: 段階的移行
1. 実行レイヤーの分離
2. データ可用性レイヤーの独立化
3. コンセンサスとセトルメントの分離

### Phase 4: 最適化と統合
1. パフォーマンス最適化
2. セキュリティ監査
3. 運用性の向上

## 技術スタック

### インターフェース通信
- **非同期通信**: Tokio + mpsc channels
- **同期通信**: 直接関数呼び出し
- **ネットワーク通信**: libp2p/TCP

### 状態管理
- **ローカル状態**: sled database
- **グローバル状態**: Merkle trie
- **キャッシュ**: LRU cache

### 設定管理
- **階層設定**: TOML config files
- **実行時設定**: Environment variables
- **動的設定**: API endpoints

## 利点

1. **スケーラビリティ**: 各レイヤーを独立してスケール
2. **モジュラリティ**: レイヤーの交換・アップグレードが容易
3. **開発効率**: チーム毎に異なるレイヤーを並行開発
4. **テスト容易性**: レイヤー毎の単体テストが可能
5. **再利用性**: 他のブロックチェーンでの利用が可能

## 次のステップ

1. 現在のコードベースのレイヤー分析
2. インターフェース設計と実装
3. 段階的なリファクタリング
4. 統合テストとベンチマーク
