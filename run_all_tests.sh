#!/bin/bash

# Script para executar TODOS os testes disponíveis

echo "=========================================="
echo "Executando TODOS os Testes"
echo "=========================================="
echo ""

# Cores para output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASSED=0
FAILED=0
TIMEOUT=0
UNKNOWN=0

run_test() {
    local rom_path=$1
    local test_name=$2

    echo -e "${YELLOW}Testando: ${test_name}${NC}"
    echo "ROM: $rom_path"

    if [ ! -f "$rom_path" ]; then
        echo -e "${RED}❌ ROM não encontrada: $rom_path${NC}"
        FAILED=$((FAILED + 1))
        return 1
    fi

    result=$(timeout 120 cargo run --release -- "$rom_path" --headless 2>&1)
    exit_code=$?

    if [ $exit_code -eq 124 ]; then
        echo -e "${YELLOW}⏱️ TIMEOUT (120s)${NC}"
        TIMEOUT=$((TIMEOUT + 1))
        return 2
    fi

    if echo "$result" | grep -q "✅ Teste passou\|Teste passou"; then
        echo -e "${GREEN}✅ PASSOU${NC}"
        PASSED=$((PASSED + 1))
        return 0
    elif echo "$result" | grep -q "❌ Teste falhou\|Failed"; then
        echo -e "${RED}❌ FALHOU${NC}"
        echo "$result" | grep -E "(Failed|❌)" | head -3
        FAILED=$((FAILED + 1))
        return 1
    elif echo "$result" | grep -q "Timeout\|timeout\|⏱️"; then
        echo -e "${YELLOW}⏱️ TIMEOUT${NC}"
        TIMEOUT=$((TIMEOUT + 1))
        return 2
    else
        echo -e "${YELLOW}⚠️ Resultado desconhecido${NC}"
        echo "$result" | tail -3
        UNKNOWN=$((UNKNOWN + 1))
        return 1
    fi
}

# Encontra todos os arquivos .gb e roda
total_tests=0
while IFS= read -r -d '' rom; do
    test_name=$(basename "$rom" .gb)
    run_test "$rom" "$test_name"
    total_tests=$((total_tests + 1))
    echo ""
done < <(find gb-test-roms -name "*.gb" -type f -print0 | sort -z)

echo "=========================================="
echo "Resumo dos Testes"
echo "=========================================="
echo -e "Total de testes: $total_tests"
echo -e "${GREEN}✅ Passou: $PASSED${NC}"
echo -e "${RED}❌ Falhou: $FAILED${NC}"
echo -e "${YELLOW}⏱️ Timeout: $TIMEOUT${NC}"
echo -e "${YELLOW}⚠️ Desconhecido: $UNKNOWN${NC}"
echo ""

if [ $FAILED -eq 0 ] && [ $TIMEOUT -eq 0 ] && [ $UNKNOWN -eq 0 ]; then
    echo -e "${GREEN}🎉 Todos os testes passaram!${NC}"
    exit 0
else
    echo -e "${RED}❌ Alguns testes falharam${NC}"
    exit 1
fi
