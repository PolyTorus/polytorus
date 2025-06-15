# Polytorus Kani 形式検証ガイド

## 概要

Polytorus ブロックチェーンでは、コードの正確性と安全性を保証するために Kani 形式検証ツールを使用しています。Kani は Rust コード用の境界モデル検証器で、プログラムの特定の性質を数学的に証明できます。

## インストール

### 前提条件

- Rust 1.70 以上
- CBMC（Kani のバックエンド）

### Kani のインストール

```bash
# Kani verifier をインストール
cargo install --locked kani-verifier

# Kani セットアップ
cargo kani setup
```

または、Makefile を使用:

```bash
make kani-install
```

## 使用方法

### 基本的な使用

```bash
# すべての検証を実行
make kani-verify

# 高速な開発用検証
make kani-quick

# 特定のカテゴリーの検証
make kani-crypto      # 暗号化機能
make kani-blockchain  # ブロックチェーン機能
make kani-modular     # モジュラーアーキテクチャ
```

### 個別のハーネス実行

```bash
# 特定の検証ハーネスを実行
cargo kani --harness verify_ecdsa_sign_verify
cargo kani --harness verify_transaction_integrity
cargo kani --harness verify_mining_stats
```

## 検証対象

### 暗号化機能

1. **ECDSA 署名検証** (`verify_ecdsa_sign_verify`)
   - 署名と検証の一貫性
   - 署名サイズの正確性

2. **暗号化タイプ判定** (`verify_encryption_type_determination`)
   - 公開鍵サイズに基づく暗号化タイプの分類
   - ECDSA vs FN-DSA の判定

3. **トランザクション整合性** (`verify_transaction_integrity`)
   - トランザクション構造の正確性
   - 入出力の妥当性検証

4. **トランザクション値の境界** (`verify_transaction_value_bounds`)
   - 値のオーバーフロー防止
   - 合計値の計算正確性

### ブロックチェーン機能

1. **マイニング統計** (`verify_mining_stats`)
   - 平均マイニング時間の計算
   - 成功率の計算

2. **マイニング試行追跡** (`verify_mining_attempts`)
   - 試行回数と成功回数の一貫性
   - 成功率の範囲チェック

3. **難易度調整設定** (`verify_difficulty_adjustment_config`)
   - 最小・最大・基本難易度の関係
   - 調整係数の範囲

4. **ブロックハッシュ一貫性** (`verify_block_hash_consistency`)
   - ハッシュの決定性
   - ブロックデータの整合性

### モジュラーアーキテクチャ

1. **メッセージ優先度順序** (`verify_message_priority_ordering`)
   - 優先度による正しいソート
   - メッセージプロパティの保持

2. **レイヤー状態遷移** (`verify_layer_state_transitions`)
   - 有効な状態遷移
   - 状態の妥当性

3. **メッセージバス容量管理** (`verify_message_bus_capacity`)
   - キューサイズの制限
   - メッセージドロップの計算

4. **オーケストレーター調整** (`verify_orchestrator_coordination`)
   - レイヤー状態の追跡
   - システム健全性チェック

## 検証結果の解釈

### 成功 (PASSED)
```
VERIFICATION:- PASSED
```
検証が成功し、指定された性質が数学的に証明されました。

### 失敗 (FAILED)
```
VERIFICATION:- FAILED
```
反例が見つかったか、検証条件が満たされませんでした。

### 不明 (UNKNOWN)
```
VERIFICATION:- UNKNOWN
```
タイムアウトまたはリソース不足により結果が不明です。

## 設定

### kani-config.toml

プロジェクトルートの `kani-config.toml` で検証設定をカスタマイズできます：

```toml
[verification.crypto_verification]
description = "暗号化操作の形式検証"
harnesses = [
    "verify_ecdsa_sign_verify",
    "verify_transaction_integrity"
]

[solver]
engine = "cbmc"
unwinding = 5

[restrictions]
function_call_limit = 100
loop_unroll = 10
```

### 境界設定

状態爆発を避けるため、シンボリック値に適切な境界を設定：

```rust
#[cfg(kani)]
#[kani::proof]
fn verify_example() {
    let value: u32 = kani::any();
    kani::assume(value <= 1000); // 境界を設定
    
    // 検証ロジック
}
```

## トラブルシューティング

### よくあるエラー

1. **タイムアウト**
   ```bash
   # より大きなタイムアウトを設定
   cargo kani --harness my_harness --timeout=600
   ```

2. **メモリ不足**
   ```bash
   # より小さな境界を設定
   kani::assume(value <= 100); // 1000 から 100 に削減
   ```

3. **状態爆発**
   - ループの展開回数を制限
   - 配列サイズを制限
   - 再帰の深さを制限

### デバッグ

```bash
# 詳細な出力で実行
cargo kani --harness my_harness -- --verbose

# 反例の表示
cargo kani --harness my_harness -- --trace
```

## CI/CD 統合

GitHub Actions での自動検証：

```yaml
- name: Run Kani Verification
  run: |
    make kani-setup
    make kani-ci  # CI 用の高速検証
```

## ベストプラクティス

1. **段階的検証**: 複雑な検証を小さな部分に分割
2. **適切な境界**: 現実的な値の範囲を設定
3. **前提条件**: `kani::assume()` で無効な入力を除外
4. **モジュラー設計**: 各モジュールに専用の検証ハーネス
5. **継続的実行**: CI/CD パイプラインに統合

## 参考資料

- [Kani 公式ドキュメント](https://model-checking.github.io/kani/)
- [CBMC ユーザーガイド](https://www.cprover.org/cbmc/)
- [Rust 形式検証ガイド](https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/cfg-kani.html)

## サポート

検証に関する問題は、プロジェクトの Issue を作成してください。
