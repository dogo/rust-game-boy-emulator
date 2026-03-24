#!/bin/bash

# Script para executar TODOS os testes Blargg disponíveis
# Usa o binário release pré-compilado para máxima velocidade

set -uo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

cleanup() {
    trap - INT TERM
    echo ""
    echo -e "${RED}🛑 Interrompido${NC}"
    exit 130
}
trap cleanup INT TERM

# Garante que o binário release existe
if [ ! -f "./target/release/gb_emu" ]; then
    echo "Compilando binário release..."
    cargo build --release --quiet
fi

BINARY="./target/release/gb_emu"

PASSED=0
FAILED=0
TIMEOUT=0

# Executa um ROM e retorna símbolo + nome
run_rom() {
    local rom="$1"
    local name
    name=$(basename "$rom" .gb)

    timeout 90 "$BINARY" --headless "$rom" >/dev/null 2>&1
    local code=$?

    case $code in
        0)   PASSED=$((PASSED+1)); echo -e "  ${GREEN}✅ $name${NC}" ;;
        1)   FAILED=$((FAILED+1)); echo -e "  ${RED}❌ $name${NC}" ;;
        2)   TIMEOUT=$((TIMEOUT+1)); echo -e "  ${YELLOW}⏱️  $name${NC}" ;;
        124) TIMEOUT=$((TIMEOUT+1)); echo -e "  ${YELLOW}⏱️  $name (timeout)${NC}" ;;
        *)   FAILED=$((FAILED+1)); echo -e "  ${RED}❌ $name (exit $code)${NC}" ;;
    esac
}

# Executa todos os ROMs de um diretório com header de seção
run_section() {
    local title="$1"
    local dir="$2"

    [ -d "$dir" ] || return

    local roms
    roms=$(find "$dir" -maxdepth 1 -name "*.gb" -type f | sort)
    [ -z "$roms" ] && return

    echo ""
    echo -e "${BOLD}$title${NC}"
    while IFS= read -r rom; do
        run_rom "$rom"
    done <<< "$roms"
}

echo "=========================================="
echo -e "${BOLD}Testes Blargg — $(date '+%H:%M:%S')${NC}"
echo "=========================================="

# cpu_instrs
run_section "cpu_instrs" "gb-test-roms/cpu_instrs/individual"

# Testes unitários simples (arquivo único)
echo ""
echo -e "${BOLD}Outros testes${NC}"
for rom in \
    "gb-test-roms/halt_bug.gb" \
    "gb-test-roms/instr_timing/instr_timing.gb" \
    "gb-test-roms/interrupt_time/interrupt_time.gb"; do
    [ -f "$rom" ] && run_rom "$rom"
done

# mem_timing e mem_timing-2
run_section "mem_timing"   "gb-test-roms/mem_timing/individual"
run_section "mem_timing-2" "gb-test-roms/mem_timing-2/rom_singles"

# oam_bug
run_section "oam_bug"      "gb-test-roms/oam_bug/rom_singles"

# dmg_sound
run_section "dmg_sound"    "gb-test-roms/dmg_sound/rom_singles"

# cgb_sound
run_section "cgb_sound"    "gb-test-roms/cgb_sound/rom_singles"

# Resumo
TOTAL=$((PASSED + FAILED + TIMEOUT))
echo ""
echo "=========================================="
echo -e "${BOLD}Resumo${NC}"
echo "=========================================="
echo -e "  Total : $TOTAL"
echo -e "  ${GREEN}✅ Passou : $PASSED${NC}"
echo -e "  ${RED}❌ Falhou : $FAILED${NC}"
echo -e "  ${YELLOW}⏱️  Timeout: $TIMEOUT${NC}"
echo ""

if [ $FAILED -eq 0 ] && [ $TIMEOUT -eq 0 ]; then
    echo -e "${GREEN}🎉 Todos os testes passaram!${NC}"
    exit 0
else
    pct=$((PASSED * 100 / TOTAL))
    echo -e "Taxa de aprovação: ${pct}%"
    exit 1
fi
