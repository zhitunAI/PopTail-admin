#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Restarting services..."
"$ROOT_DIR/dev-stop.sh"
"$ROOT_DIR/dev-up.sh"
