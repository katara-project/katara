#!/usr/bin/env bash
set -euo pipefail

echo ""
echo "  DISTIRA — The AI Context Compiler"
echo "  Bootstrap script for Linux / macOS"
echo "  ──────────────────────────────────────────────"
echo ""

warnings=()

# ── 1. Check Rust ──────────────────────────────────────
if command -v cargo &>/dev/null; then
  echo "[ok] Rust $(rustc --version | awk '{print $2}')"
else
  echo "[!!] Rust not found. Installing via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck source=/dev/null
  source "$HOME/.cargo/env"
fi

# ── 2. Check Node.js (>= 20 required for MCP server) ──
if command -v node &>/dev/null; then
  node_ver=$(node --version | sed 's/^v//')
  node_major=$(echo "$node_ver" | cut -d. -f1)
  if [ "$node_major" -ge 20 ]; then
    echo "[ok] Node.js v$node_ver"
  else
    echo "[!!] Node.js v$node_ver detected but >= 20 required (MCP server needs native fetch)."
    exit 1
  fi
else
  echo "[!!] Node.js not found. Please install Node.js 20+ from https://nodejs.org"
  exit 1
fi

# ── 3. Check Ollama (optional but recommended) ────────
if command -v ollama &>/dev/null; then
  echo "[ok] Ollama: $(ollama --version 2>&1)"
else
  warnings+=("Ollama not found. Install from https://ollama.com/download for local model routing.")
  echo "[--] Ollama not found (optional — needed for local models)."
fi

# ── 4. Verify config files exist ──────────────────────
echo ""
echo "==> Checking configuration files..."
for cfg in configs/providers/providers.yaml configs/routing/routing.yaml configs/policies/policies.yaml; do
  if [ -f "$cfg" ]; then
    echo "[ok] $cfg"
  else
    echo "[!!] Missing: $cfg"
    warnings+=("Configuration file missing: $cfg — see INSTALL.md for examples.")
  fi
done

# ── 5. Create .env from .env.example if absent ────────
if [ ! -f .env ]; then
  if [ -f .env.example ]; then
    cp .env.example .env
    echo "[ok] Created .env from .env.example (edit it to add your API keys)."
    warnings+=(".env created from template — edit it to set your real API keys before using cloud providers.")
  else
    echo "[--] No .env.example found, skipping .env creation."
  fi
else
  echo "[ok] .env already exists."
fi

# ── 6. Build Rust workspace ───────────────────────────
echo ""
echo "==> Building Rust workspace (8 crates)..."
cargo build --workspace
echo "[ok] Rust build complete."

# ── 7. Install dashboard dependencies ─────────────────
echo ""
echo "==> Installing dashboard dependencies..."
pushd dashboard/ui-vue > /dev/null
npm install
popd > /dev/null
echo "[ok] Dashboard dependencies installed."

# ── 8. Pull Ollama models (if Ollama is available) ────
if command -v ollama &>/dev/null && [ -f configs/providers/providers.yaml ]; then
  echo ""
  echo "==> Checking Ollama models declared in providers.yaml..."

  # Get list of already-installed models
  installed=$(ollama list 2>/dev/null | tail -n +2 | awk '{print $1}' || true)

  grep -oP '^\s+model:\s+\K.+' configs/providers/providers.yaml \
    | grep -v 'mistral-ocr' \
    | sort -u \
    | while read -r model; do
        # Check if model is already installed (exact match or prefix match with tag)
        if echo "$installed" | grep -qiE "^${model}(:|$)"; then
          echo "  [ok] $model (already installed)"
        else
          echo "  Pulling $model ..."
          ollama pull "$model"
          echo "  [ok] $model"
        fi
      done
fi

# ── 9. Verify MCP server module syntax ────────────────
echo ""
echo "==> Verifying MCP server module..."
if node --check mcp/distira-server.mjs 2>/dev/null; then
  echo "[ok] MCP server (mcp/distira-server.mjs) syntax OK."
else
  echo "[!!] MCP server failed to load — check mcp/distira-server.mjs."
  warnings+=("MCP server module failed to load.")
fi

# ── Summary ───────────────────────────────────────────
echo ""
echo "  ──────────────────────────────────────────────"
echo "  DISTIRA bootstrap complete!"
echo ""

if [ ${#warnings[@]} -gt 0 ]; then
  echo "  Warnings:"
  for w in "${warnings[@]}"; do
    echo "    - $w"
  done
  echo ""
fi

echo "  Next steps:"
echo "    Start everything:  ./scripts/start.sh"
echo "    Or manually:"
echo "      1. ollama serve"
echo "      2. cargo run -p core"
echo "      3. cd dashboard/ui-vue && npm run dev"
echo "      4. @distira in Copilot Chat"
echo ""
