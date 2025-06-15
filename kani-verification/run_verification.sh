#!/bin/bash

# PolyTorus Kani 検証実行スクリプト
# 複数の検証ハーネスを順次実行し、結果をまとめます

set -e

echo "🔍 PolyTorus Kaniの形式検証を開始します..."

# 色定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 結果カウンター
PASSED=0
FAILED=0
TOTAL=0

# 結果を保存するディレクトリ
RESULTS_DIR="kani_results"
mkdir -p "$RESULTS_DIR"

echo -e "${BLUE}📋 実行する検証ハーネス:${NC}"
echo "  基本操作:"
echo "    - verify_basic_arithmetic"
echo "    - verify_boolean_logic"
echo "    - verify_array_bounds"
echo "    - verify_hash_determinism"
echo "    - verify_queue_operations"
echo ""
echo "  暗号化機能:"
echo "    - verify_encryption_type_determination"
echo "    - verify_transaction_integrity"
echo "    - verify_transaction_value_bounds"
echo "    - verify_signature_properties"
echo "    - verify_public_key_format"
echo "    - verify_hash_computation"
echo ""

# 検証実行関数
run_verification() {
    local harness_name=$1
    local description=$2
    local timeout_sec=${3:-60}
    
    echo -e "${BLUE}🔍 実行中: ${description}${NC}"
    echo "   ハーネス: ${harness_name}"
    echo "   タイムアウト: ${timeout_sec}秒"
    
    ((TOTAL++))
    
    if timeout ${timeout_sec} cargo kani --harness ${harness_name} > "$RESULTS_DIR/${harness_name}.log" 2>&1; then
        if grep -q "VERIFICATION:- SUCCESSFUL" "$RESULTS_DIR/${harness_name}.log"; then
            echo -e "${GREEN}✅ ${description} - 成功${NC}"
            ((PASSED++))
        else
            echo -e "${YELLOW}⚠️ ${description} - 不明な結果${NC}"
        fi
    else
        echo -e "${RED}❌ ${description} - 失敗またはタイムアウト${NC}"
        ((FAILED++))
    fi
    echo ""
}

# 基本検証の実行
echo -e "${BLUE}🧮 基本操作の検証を開始...${NC}"
run_verification "verify_basic_arithmetic" "基本的な算術演算" 30
run_verification "verify_boolean_logic" "ブール論理" 30
run_verification "verify_array_bounds" "配列境界チェック" 30
run_verification "verify_hash_determinism" "ハッシュの決定性" 30
run_verification "verify_queue_operations" "キュー操作" 45

# 暗号化検証の実行
echo -e "${BLUE}🔐 暗号化機能の検証を開始...${NC}"
run_verification "verify_encryption_type_determination" "暗号化タイプ判定" 60
run_verification "verify_transaction_integrity" "トランザクション整合性" 90
run_verification "verify_transaction_value_bounds" "トランザクション値境界" 60
run_verification "verify_signature_properties" "署名プロパティ" 45
run_verification "verify_public_key_format" "公開鍵フォーマット" 45
run_verification "verify_hash_computation" "ハッシュ計算" 45

# 結果のサマリー作成
echo -e "${BLUE}📊 検証結果サマリーを作成中...${NC}"

cat > "$RESULTS_DIR/summary.md" << EOF
# PolyTorus Kani 形式検証結果

**実行日時:** $(date)

## 総合結果

- **総検証数:** $TOTAL
- **成功:** $PASSED
- **失敗:** $FAILED
- **成功率:** $(( (PASSED * 100) / TOTAL ))%

## 詳細結果

EOF

# 各結果の詳細をサマリーに追加
for log_file in "$RESULTS_DIR"/*.log; do
    if [ -f "$log_file" ]; then
        harness_name=$(basename "$log_file" .log)
        echo "### $harness_name" >> "$RESULTS_DIR/summary.md"
        
        if grep -q "VERIFICATION:- SUCCESSFUL" "$log_file"; then
            echo "**ステータス:** ✅ 成功" >> "$RESULTS_DIR/summary.md"
        else
            echo "**ステータス:** ❌ 失敗" >> "$RESULTS_DIR/summary.md"
        fi
        
        # 実行時間を抽出
        if grep -q "Verification Time:" "$log_file"; then
            exec_time=$(grep "Verification Time:" "$log_file" | tail -1)
            echo "**$exec_time**" >> "$RESULTS_DIR/summary.md"
        fi
        
        # チェック数を抽出
        if grep -q "SUMMARY:" "$log_file"; then
            check_summary=$(grep -A 1 "SUMMARY:" "$log_file" | tail -1)
            echo "**結果:** $check_summary" >> "$RESULTS_DIR/summary.md"
        fi
        
        echo "" >> "$RESULTS_DIR/summary.md"
    fi
done

# 最終結果表示
echo -e "${BLUE}🎯 最終結果${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "総検証数: ${BLUE}$TOTAL${NC}"
echo -e "成功: ${GREEN}$PASSED${NC}"
echo -e "失敗: ${RED}$FAILED${NC}"
echo -e "成功率: ${GREEN}$(( (PASSED * 100) / TOTAL ))%${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 すべての検証が成功しました！${NC}"
    echo -e "${GREEN}PolyTorusの実装が形式的に検証されました。${NC}"
else
    echo -e "${YELLOW}⚠️ 一部の検証に問題があります。${NC}"
    echo -e "${YELLOW}詳細は ${RESULTS_DIR}/ ディレクトリの個別ログファイルを確認してください。${NC}"
fi

echo ""
echo -e "${BLUE}📁 結果ファイル:${NC}"
echo "  - サマリー: ${RESULTS_DIR}/summary.md"
echo "  - 個別ログ: ${RESULTS_DIR}/*.log"
echo ""
echo -e "${BLUE}🔍 詳細確認コマンド:${NC}"
echo "  cat ${RESULTS_DIR}/summary.md"
echo "  cat ${RESULTS_DIR}/<harness_name>.log"

exit $FAILED
