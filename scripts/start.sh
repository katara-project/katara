#!/usr/bin/env bash
#
# Start all KATARA services (Ollama, backend, dashboard) in one command.
# Each service runs in the background. Press Ctrl+C to stop everything.
#
# Usage: ./scripts/start.sh
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

echo ""
echo "  KATARA — Starting all services"
echo "  ──────────────────────────────"
echo ""

PIDS=()

# ── Cleanup on exit ───────────────────────────────────
cleanup() {
    echo ""
    echo "  Stopping all KATARA services..."
    for pid in "${PIDS[@]}"; do
        kill "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
    done
    echo "  All services stopped."
}
trap cleanup EXIT INT TERM

# ── Load .env secrets ─────────────────────────────────
if [ -f .env ]; then
    set -a
    # shellcheck source=/dev/null
    source .env
    set +a
    echo "[ok] .env loaded."
else
    echo "[--] No .env file found (cloud providers will have no API keys)."
fi

# ── 1. Start Ollama ───────────────────────────────────
ollama_running=false
if command -v ollama &>/dev/null; then
    if curl -sf http://localhost:11434/api/tags >/dev/null 2>&1; then
        echo "[ok] Ollama is already running."
        ollama_running=true
    else
        echo "==> Starting Ollama..."
        ollama serve &>/dev/null &
        PIDS+=($!)

        # Wait for Ollama to be ready (max 30s)
        for i in $(seq 1 30); do
            if curl -sf http://localhost:11434/api/tags >/dev/null 2>&1; then
                ollama_running=true
                break
            fi
            sleep 1
        done

        if $ollama_running; then
            echo "[ok] Ollama started on :11434"
        else
            echo "[!!] Ollama failed to start within 30s."
        fi
    fi
else
    echo "[--] Ollama not installed — skipping local models."
fi

# ── 2. Start KATARA backend ───────────────────────────
echo "==> Starting KATARA backend..."
cargo run -p core &
PIDS+=($!)

# Wait for backend to be ready (max 120s — includes compile time)
backend_ready=false
for i in $(seq 1 120); do
    if curl -sf http://localhost:8080/healthz >/dev/null 2>&1; then
        backend_ready=true
        break
    fi
    sleep 1
done

if $backend_ready; then
    echo "[ok] KATARA backend running on http://127.0.0.1:8080"
else
    echo "[..] Backend still starting (compilation may take a moment)..."
fi

# ── 3. Start Vue dashboard ────────────────────────────
echo "==> Starting dashboard..."
(cd dashboard/ui-vue && npm run dev) &
PIDS+=($!)
sleep 3
echo "[ok] Dashboard starting on http://localhost:5173"

# ── Summary ───────────────────────────────────────────
echo ""
echo "  ──────────────────────────────────────────────"
echo "  All KATARA services launched!"
echo ""
echo "  Services:"
if $ollama_running; then
    echo "    Ollama        → http://localhost:11434"
fi
echo "    KATARA API    → http://localhost:8080"
echo "    Dashboard     → http://localhost:5173"
echo "    VS Code Agent → @katara in Copilot Chat"
echo ""
echo "  Press Ctrl+C to stop all services."
echo ""

# ── Keep alive — wait for backend ─────────────────────
wait "${PIDS[1]}" 2>/dev/null || true
