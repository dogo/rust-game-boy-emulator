#!/usr/bin/env bash

# Script para executar ROMs de teste em modo headless.
# Recompila incrementalmente o binário release antes de testar.

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
ERRORS=0
EXPECTED_BLARGG=52
EXPECTED_MOONEYE=111

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

# O Cargo só recompila o que mudou; falha de build interrompe a suíte.
if ! cargo build --release --quiet; then
    echo -e "${RED}❌ Falha ao compilar o emulador${NC}" >&2
    exit 2
fi

ROM_OUTPUT="$(mktemp)"
trap 'rm -f "$ROM_OUTPUT"' EXIT

require_dir() {
    if [ ! -d "$1" ]; then
        echo "Diretório de ROMs não encontrado: $1" >&2
        exit 2
    fi
}

require_file() {
    if [ ! -f "$1" ]; then
        echo "ROM de teste não encontrada: $1" >&2
        exit 2
    fi
}

# Modelo de hardware pelas convenções de nome das ROMs de teste (Mooneye:
# -dmg0, -mgb, -S, -A...). Só a suíte faz isso; o emulador recebe --model.
boot_model_for() {
    case "${1,,}" in
        *-dmg0*) echo dmg0 ;;
        *-mgb*) echo mgb ;;
        *-sgb2* | *boot_div2-s*) echo sgb2 ;;
        *-sgb* | *-s.gb) echo sgb ;;
        *-cgb0*) echo cgb0 ;;
        *-cgb*) echo cgb ;;
        *-a.gb) echo agb ;;
        *-c.gb) echo cgb ;;
        *)
            # ROMs só CGB sem sufixo no nome (cgb_sound, interrupt_time): flag $C0 em $0143
            if [ "$(od -An -tx1 -j323 -N1 "$1" | tr -d ' ')" = "c0" ]; then
                echo cgb
            fi
            ;;
    esac
}

# Executa um ROM e retorna símbolo + nome
run_rom() {
    local rom="$1"
    local name
    name="${rom#*/}"
    name="${name%.gb}"

    local model_args=()
    local model
    model="$(boot_model_for "$rom")"
    if [ -n "$model" ]; then
        model_args=(--model "$model")
    fi

    timeout 90 "$BINARY" --headless "${model_args[@]}" "$rom" >"$ROM_OUTPUT" 2>&1
    local code=$?

    case $code in
        0)   PASSED=$((PASSED+1)); echo -e "  ${GREEN}✅ $name${NC}" ;;
        1)   FAILED=$((FAILED+1)); echo -e "  ${RED}❌ $name${NC}" ;;
        2)   TIMEOUT=$((TIMEOUT+1)); echo -e "  ${YELLOW}⏱️ $name${NC}" ;;
        124) TIMEOUT=$((TIMEOUT+1)); echo -e "  ${YELLOW}⏱️ $name (timeout)${NC}" ;;
        *)   ERRORS=$((ERRORS+1)); echo -e "  ${RED}⚠️ $name (erro de execução $code)${NC}" ;;
    esac
    if [ "$code" -ne 0 ]; then
        tail -n 30 "$ROM_OUTPUT"
    fi
}

# Executa todos os ROMs de um diretório com header de seção.
run_section_flat() {
    local title="$1"
    local dir="$2"

    require_dir "$dir"

    local roms
    roms=$(find "$dir" -maxdepth 1 -name "*.gb" -type f | sort)
    if [ -z "$roms" ]; then
        echo "Nenhuma ROM de teste em: $dir" >&2
        exit 2
    fi

    echo ""
    echo -e "${BOLD}$title${NC}"
    while IFS= read -r rom; do
        run_rom "$rom"
    done <<< "$roms"
}

run_section_recursive() {
    local title="$1"
    local dir="$2"

    require_dir "$dir"

    local roms
    roms=$(find "$dir" -name "*.gb" -type f | sort)
    if [ -z "$roms" ]; then
        echo "Nenhuma ROM de teste em: $dir" >&2
        exit 2
    fi

    echo ""
    echo -e "${BOLD}$title${NC}"
    while IFS= read -r rom; do
        run_rom "$rom"
    done <<< "$roms"
}

run_blargg() {
    echo ""
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
        require_file "$rom"
        run_rom "$rom"
    done

    run_section_flat "mem_timing"   "gb-test-roms/mem_timing/individual"
    run_section_flat "mem_timing-2" "gb-test-roms/mem_timing-2/rom_singles"
    run_section_flat "oam_bug"      "gb-test-roms/oam_bug/rom_singles"
    run_section_flat "dmg_sound"    "gb-test-roms/dmg_sound/rom_singles"
    run_section_flat "cgb_sound"    "gb-test-roms/cgb_sound/rom_singles"
}

run_mooneye() {
    require_dir "mooneye-roms"
    echo ""
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

TOTAL=$((PASSED + FAILED + TIMEOUT + ERRORS))
case "$SUITE" in
    blargg) EXPECTED_TOTAL=$EXPECTED_BLARGG ;;
    mooneye) EXPECTED_TOTAL=$EXPECTED_MOONEYE ;;
    all) EXPECTED_TOTAL=$((EXPECTED_BLARGG + EXPECTED_MOONEYE)) ;;
esac
echo ""
echo "=========================================="
echo -e "${BOLD}Resumo${NC}"
echo "=========================================="
echo -e "  Total : $TOTAL"
echo -e "  ${GREEN}✅ Passou : $PASSED${NC}"
echo -e "  ${RED}❌ Falhou : $FAILED${NC}"
echo -e "  ${YELLOW}⏱️ Timeout: $TIMEOUT${NC}"
echo -e "  ${RED}⚠️ Erros  : $ERRORS${NC}"
echo ""

if [ "$TOTAL" -ne "$EXPECTED_TOTAL" ]; then
    echo -e "${RED}Esperadas $EXPECTED_TOTAL ROMs, executadas $TOTAL. Verifique as fixtures.${NC}" >&2
    exit 2
fi

if [ "$ERRORS" -gt 0 ]; then
    exit 2
fi

if [ "$FAILED" -eq 0 ] && [ "$TIMEOUT" -eq 0 ]; then
    echo -e "${GREEN}🎉 Todos os testes passaram!${NC}"
    exit 0
else
    pct=0
    [ $TOTAL -gt 0 ] && pct=$((PASSED * 100 / TOTAL))
    echo -e "Taxa de aprovação: ${pct}%"
    exit 1
fi
