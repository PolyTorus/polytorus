# PolyTorus TPS Benchmark Analysis

## 概要
このドキュメントでは、PolyTorusブロックチェーンのTPS（Transactions Per Second）ベンチマーク結果の分析方法について説明します。

## ベンチマーク種類

### 1. TPS Throughput Benchmark (`benchmark_tps`)
**目的**: マイニングと検証を含む実世界のTPSパフォーマンスを測定

**測定項目**:
- 10, 25, 50トランザクションでの処理速度
- ブロック作成、マイニング、検証の全プロセス
- 低難易度設定での最適化されたパフォーマンス

**重要な指標**:
- 平均処理時間 (Mean time)
- 標準偏差 (Standard deviation)
- スループット (Transactions/second)

### 2. Pure Transaction Processing (`benchmark_pure_transaction_processing`)
**目的**: マイニングなしの純粋なトランザクション処理速度を測定

**測定項目**:
- 50, 100, 500トランザクションでの処理速度
- トランザクション作成と基本検証のみ
- 理論的最大TPS

### 3. Concurrent TPS (`benchmark_concurrent_tps`)
**目的**: マルチスレッド環境でのTPS性能を測定

**測定項目**:
- 2, 4スレッドでの並列処理
- スレッド間での処理効率
- スケーラビリティの評価

## 結果の分析方法

### 1. HTML報告書の確認
```bash
# ブラウザで詳細結果を確認
firefox target/criterion/report/index.html
```

### 2. TPS計算
```
実効TPS = トランザクション数 / 処理時間(秒)
```

### 3. パフォーマンス比較
- **Baseline TPS**: Pure transaction processingの結果
- **Real-world TPS**: TPS throughputの結果  
- **Concurrent efficiency**: (並列TPS / 単一スレッドTPS) / スレッド数

## 最適化の指針

### 高TPS化のアプローチ
1. **トランザクション検証の最適化**
   - 署名検証の並列化
   - UTXO検索の高速化

2. **マイニング効率の向上**
   - 適応的難易度調整
   - ハッシュ計算の最適化

3. **メモリ使用量の削減**
   - 効率的なデータ構造
   - キャッシング戦略

4. **並列処理の改善**
   - ロックフリーデータ構造
   - ワーカープールの活用

## ベンチマーク実行コマンド

```bash
# 全TPSベンチマーク実行
./benchmark_tps.sh

# 個別ベンチマーク実行
cargo bench --bench blockchain_bench benchmark_tps
cargo bench --bench blockchain_bench benchmark_pure_transaction_processing
cargo bench --bench blockchain_bench benchmark_concurrent_tps

# 比較ベンチマーク（ベースライン保存）
cargo bench --bench blockchain_bench -- --save-baseline before_optimization

# 最適化後の比較
cargo bench --bench blockchain_bench -- --baseline before_optimization
```

## 期待値とベンチマーク目標

### 開発環境での目標TPS
- **Pure transaction processing**: 1,000+ TPS
- **With mining (low difficulty)**: 100+ TPS  
- **Production scenario**: 10-50 TPS

### パフォーマンス改善の指標
- 10%以上の向上で有意な改善
- レグレッション検出: 5%以上の低下で警告

## トラブルシューティング

### よくある問題
1. **メモリ不足**: 大量トランザクションでのOOM
2. **CPU使用率**: マイニング処理での高負荷
3. **I/O待機**: データベース操作のボトルネック

### 対処方法
```bash
# メモリ使用量制限
ulimit -m 2097152  # 2GB制限

# CPU使用率監視
htop &
cargo bench --bench blockchain_bench

# I/O監視
iotop &
```
