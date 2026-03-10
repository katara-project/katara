#!/usr/bin/env bash
set -euo pipefail

echo "==> Bootstrapping KATARA..."

# 1. Check Rust
if command -v cargo &>/dev/null; then
  echo "[ok] Rust $(rustc --version | awk '{print $2}')"
else
  echo "[!!] Rust not found. Installing via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck source=/dev/null
  source "$HOME/.cargo/env"
fi

# 2. Check Node.js
if command -v node &>/dev/null; then
  echo "[ok] Node.js $(node --version)"
else
  echo "[!!] Node.js not found. Please install Node.js 20+ from https://nodejs.org"
  exit 1
fi

# 3. Build Rust workspace
echo "==> Building Rust workspace..."
cargo build --workspace
echo "[ok] Rust build complete."

# 4. Install dashboard dependencies
echo "==> Installing dashboard dependencies..."
cd dashboard/ui-vue
npm install
echo "[ok] Dashboard dependencies installed."

echo ""
echo "==> KATARA bootstrap complete."
echo "    Run: cargo run -p core       (gateway on :8080)"
echo "    Run: npm run dev             (dashboard on :5173)"
