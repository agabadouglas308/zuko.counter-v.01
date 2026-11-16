#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT="${ROOT}/cuassensors"

g++ -std=c++17 -Wall -Wextra -Wpedantic "${ROOT}/src/main.cpp" -o "${OUT}"
"${OUT}"
