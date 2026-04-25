#!/usr/bin/env bash

# Script para executar ROMs de teste em modo headless.
# Usa o binário release pré-compilado para máxima velocidade.

set -uo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

SUITE="${1:-blargg}"
BINARY="./target/release/gb_emu"
PASSED=0
FAILED=0
TIMEOUT=0

usage() {
    cat <<EOF
Uso: $0 [blargg|mooneye|all]

  blargg   Executa os testes Blargg versionados em gb-test-roms/ (padrão)
  mooneye  Executa os testes Mooneye automatizáveis em mooneye-roms/
  all      Executa Blargg e Mooneye

Antes de rodar Mooneye pela primeira vez:
  ./scripts/fetch_mooneye_roms.sh
EOF
}

cleanup() {
    trap - INT TERM
    echo ""
    echo -e "${RED}🛑 Interrompido${NC}"
    exit 130
}
trap cleanup INT TERM

case "$SUITE" in
    blargg|mooneye|all) ;;
    -h|--help|help)
        usage
        exit 0
        ;;
    *)
        usage >&2
        exit 2
        ;;
esac

# Garante que o binário release existe
if [ ! -f "./target/release/gb_emu" ]; then
    echo "Compilando binário release..."
    cargo build --release --quiet
fi

# Executa um ROM e retorna símbolo + nome
run_rom() {
    local rom="$1"
    local name
    name="${rom#*/}"
    name="${name%.gb}"

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

# Executa todos os ROMs de um diretório com header de seção.
run_section_flat() {
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

run_section_recursive() {
    local title="$1"
    local dir="$2"

    [ -d "$dir" ] || return

    local roms
    roms=$(find "$dir" -name "*.gb" -type f | sort)
    [ -z "$roms" ] && return

    echo ""
    echo -e "${BOLD}$title${NC}"
    while IFS= read -r rom; do
        run_rom "$rom"
    done <<< "$roms"
}

run_blargg() {
    echo "=========================================="
    echo -e "${BOLD}Testes Blargg — $(date '+%H:%M:%S')${NC}"
    echo "=========================================="

    run_section_flat "cpu_instrs" "gb-test-roms/cpu_instrs/individual"

    echo ""
    echo -e "${BOLD}Outros testes${NC}"
    for rom in \
        "gb-test-roms/halt_bug.gb" \
        "gb-test-roms/instr_timing/instr_timing.gb" \
        "gb-test-roms/interrupt_time/interrupt_time.gb"; do
        [ -f "$rom" ] && run_rom "$rom"
    done

    run_section_flat "mem_timing"   "gb-test-roms/mem_timing/individual"
    run_section_flat "mem_timing-2" "gb-test-roms/mem_timing-2/rom_singles"
    run_section_flat "oam_bug"      "gb-test-roms/oam_bug/rom_singles"
    run_section_flat "dmg_sound"    "gb-test-roms/dmg_sound/rom_singles"
    run_section_flat "cgb_sound"    "gb-test-roms/cgb_sound/rom_singles"
}

run_mooneye() {
    if [ ! -d "mooneye-roms" ]; then
        echo "Diretório mooneye-roms/ não encontrado." >&2
        echo "Rode: ./scripts/fetch_mooneye_roms.sh" >&2
        exit 2
    fi

    echo "=========================================="
    echo -e "${BOLD}Testes Mooneye — $(date '+%H:%M:%S')${NC}"
    echo "=========================================="

    run_section_recursive "acceptance" "mooneye-roms/acceptance"
    run_section_recursive "emulator-only" "mooneye-roms/emulator-only"
    run_section_recursive "misc" "mooneye-roms/misc"
}

case "$SUITE" in
    blargg)
        run_blargg
        ;;
    mooneye)
        run_mooneye
        ;;
    all)
        run_blargg
        run_mooneye
        ;;
esac

TOTAL=$((PASSED + FAILED + TIMEOUT))
echo ""
echo "=========================================="
echo -e "${BOLD}Resumo${NC}"
echo "=========================================="
echo -e "  Total : $TOTAL"
echo -e "  ${GREEN}✅ Passou : $PASSED${NC}"
echo -e "  ${RED}❌ Falhou : $FAILED${NC}"
echo -e "  ${YELLOW}⏱️ Timeout: $TIMEOUT${NC}"
echo ""

if [ $FAILED -eq 0 ] && [ $TIMEOUT -eq 0 ]; then
    echo -e "${GREEN}🎉 Todos os testes passaram!${NC}"
    exit 0
else
    pct=0
    [ $TOTAL -gt 0 ] && pct=$((PASSED * 100 / TOTAL))
    echo -e "Taxa de aprovação: ${pct}%"
    exit 1
fi
