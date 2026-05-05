#!/usr/bin/env bash
# Setup script for the redforge subtree.
#
# Provisions everything a fresh user needs to run a smoke test against any
# of the three frameworks. By default installs only the Python deps (which
# is enough for the matrix-summary tooling and the figure scripts);
# pass `--with-<framework>` flags to bring up framework-specific bits.
#
# Usage:
#   ./scripts/setup.sh                              # core only
#   ./scripts/setup.sh --with-hermes                # + Hermes vendored
#   ./scripts/setup.sh --with-ironclaw              # + cargo build of bench
#   ./scripts/setup.sh --with-openclaw              # + openclaw docker image
#   ./scripts/setup.sh --with-all                   # everything
#
# Re-runnable: each step checks for the artifact it produces and skips if
# already present (use `--rebuild` to force).

set -euo pipefail

WITH_HERMES=0
WITH_IRONCLAW=0
WITH_OPENCLAW=0
REBUILD=0
for arg in "$@"; do
  case "$arg" in
    --with-hermes)   WITH_HERMES=1 ;;
    --with-ironclaw) WITH_IRONCLAW=1 ;;
    --with-openclaw) WITH_OPENCLAW=1 ;;
    --with-all)      WITH_HERMES=1; WITH_IRONCLAW=1; WITH_OPENCLAW=1 ;;
    --rebuild)       REBUILD=1 ;;
    *) echo "unknown arg: $arg" >&2; exit 1 ;;
  esac
done

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$(cd "$ROOT/.." && pwd)"
cd "$ROOT"

# ─── Python venv + core deps ────────────────────────────────────────────────
if [[ ! -d .venv || $REBUILD -eq 1 ]]; then
  echo "==> creating .venv"
  python3 -m venv .venv
fi
# shellcheck source=/dev/null
source .venv/bin/activate
echo "==> installing core requirements"
pip install --quiet -r requirements.txt

# ─── Hermes (Python) ────────────────────────────────────────────────────────
if [[ $WITH_HERMES -eq 1 ]]; then
  HERMES_DIR="$ROOT/vendor/hermes"
  if [[ ! -d "$HERMES_DIR" || $REBUILD -eq 1 ]]; then
    echo "==> cloning NousResearch/hermes-agent into vendor/hermes"
    rm -rf "$HERMES_DIR"
    git clone --depth 1 https://github.com/NousResearch/hermes-agent.git "$HERMES_DIR"
  fi
  echo "==> installing Hermes deps + framework deps"
  pip install --quiet -r requirements-frameworks.txt
  # The harness expects `run_agent` importable from PYTHONPATH; setup.sh
  # users should add this to their env:
  echo "==> Hermes ready. To use, set:  export PYTHONPATH=$HERMES_DIR:."
fi

# ─── Ironclaw bench binary (Rust) ───────────────────────────────────────────
if [[ $WITH_IRONCLAW -eq 1 ]]; then
  # The cargo build pulls in `openssl-sys`, which needs pkg-config and the
  # OpenSSL headers present on the box. macOS toolchains ship these by
  # default; Linux distros generally do not. Pre-flight check rather than
  # silent sudo install — easier to audit and to translate across distros.
  if [[ "$(uname -s)" == "Linux" ]] && ! pkg-config --exists openssl 2>/dev/null; then
    echo "ERROR: missing OpenSSL build deps required by cargo's openssl-sys crate." >&2
    echo "       On Debian/Ubuntu:  sudo apt install pkg-config libssl-dev" >&2
    echo "       On Fedora/RHEL:    sudo dnf install pkgconf-pkg-config openssl-devel" >&2
    echo "       On Arch:           sudo pacman -S pkgconf openssl" >&2
    exit 1
  fi

  if ! command -v cargo >/dev/null 2>&1; then
    echo "ERROR: cargo not on PATH. Install Rust via https://rustup.rs and retry." >&2
    exit 1
  fi

  echo "==> building nearai-bench (cargo --release)"
  pushd "$REPO_ROOT" >/dev/null
  if [[ ! -x target/release/nearai-bench || $REBUILD -eq 1 ]]; then
    cargo build --release --bin nearai-bench
  else
    echo "    target/release/nearai-bench already built (use --rebuild to force)"
  fi
  popd >/dev/null
  echo "==> Ironclaw bench binary at: $REPO_ROOT/target/release/nearai-bench"
fi

# ─── Openclaw docker image ──────────────────────────────────────────────────
if [[ $WITH_OPENCLAW -eq 1 ]]; then
  if ! command -v docker >/dev/null 2>&1; then
    echo "ERROR: docker not installed. Install Docker Desktop or docker-ce." >&2
    exit 1
  fi
  if ! docker image inspect openclaw:local >/dev/null 2>&1 || [[ $REBUILD -eq 1 ]]; then
    echo "==> building openclaw:local image"
    OPENCLAW_DIR="$ROOT/vendor/openclaw"
    if [[ ! -d "$OPENCLAW_DIR" || $REBUILD -eq 1 ]]; then
      rm -rf "$OPENCLAW_DIR"
      git clone --depth 1 https://github.com/nearai/openclaw.git "$OPENCLAW_DIR"
    fi
    docker build -t openclaw:local "$OPENCLAW_DIR"
  else
    echo "==> openclaw:local image already present (use --rebuild to force)"
  fi
fi

echo
echo "=== setup complete ==="
[[ $WITH_HERMES   -eq 1 ]] && echo "Hermes vendored at: $ROOT/vendor/hermes"
[[ $WITH_IRONCLAW -eq 1 ]] && echo "Bench binary at:    $REPO_ROOT/target/release/nearai-bench"
[[ $WITH_OPENCLAW -eq 1 ]] && echo "Openclaw image:     openclaw:local"
echo
echo "Run scripts/smoke_test.sh next (set FRAMEWORK / MODEL env vars to pick)."
