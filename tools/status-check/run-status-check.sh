#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_JSON="${ROOT_DIR}/docs/LESSJS_DIFF_REPORT.json"
WITH_PERF=0
STRICT_MAPPINGS=1
OBSERVE_MAPPINGS=0

for arg in "$@"; do
  case "$arg" in
    --with-perf)
      WITH_PERF=1
      ;;
    --strict-mappings)
      STRICT_MAPPINGS=1
      OBSERVE_MAPPINGS=0
      ;;
    --observe-mappings|--no-strict-mappings)
      STRICT_MAPPINGS=0
      OBSERVE_MAPPINGS=1
      ;;
    -h|--help)
      cat <<'EOF'
Usage: bash tools/status-check/run-status-check.sh [--with-perf] [--strict-mappings|--observe-mappings]

Options:
  --with-perf           Also run performance regression gate (cargo bench + baseline compare)
  --strict-mappings     Strict source map mappings gate (default)
  --observe-mappings    Observe mappings diff without failing (alias: --no-strict-mappings)
EOF
      exit 0
      ;;
    *)
      echo "[status-check] unknown argument: $arg" >&2
      exit 1
      ;;
  esac
done

cd "${ROOT_DIR}"

echo "[status-check] 1/5 cargo test --quiet"
cargo test --quiet

echo "[status-check] 2/5 cargo test --all-features --quiet"
cargo test --all-features --quiet

echo "[status-check] 3/5 cargo clippy --all-targets --all-features -- -D warnings"
cargo clippy --all-targets --all-features -- -D warnings

LESSJS_ARGS=(tools/lessjs-compat/run-lessjs-compat.js --strict)
if [[ "${STRICT_MAPPINGS}" -eq 1 ]]; then
  LESSJS_ARGS+=(--strict-mappings)
fi
if [[ "${OBSERVE_MAPPINGS}" -eq 1 ]]; then
  LESSJS_ARGS+=(--observe-mappings)
fi

echo "[status-check] 4/5 node ${LESSJS_ARGS[*]}"
node "${LESSJS_ARGS[@]}"

echo "[status-check] 5/5 validate docs/LESSJS_DIFF_REPORT.json summary"
node - "${REPORT_JSON}" <<'NODE'
const fs = require("fs");

const reportPath = process.argv[2];
if (!reportPath || !fs.existsSync(reportPath)) {
  console.error(`[status-check] missing report json: ${reportPath || "<empty>"}`);
  process.exit(1);
}

let report;
try {
  report = JSON.parse(fs.readFileSync(reportPath, "utf8"));
} catch (err) {
  console.error(`[status-check] failed to parse report json: ${err.message}`);
  process.exit(1);
}

const summary = report && report.summary ? report.summary : {};
const fail = Number(summary.fail || 0);
const blocked = Number(summary.blocked || 0);

if (fail !== 0 || blocked !== 0) {
  console.error(
    `[status-check] less.js report gate failed: fail=${fail}, blocked=${blocked}`
  );
  process.exit(1);
}

console.log(
  `[status-check] less.js report gate passed: fail=${fail}, blocked=${blocked}`
);
NODE

echo "[status-check] core checks passed"

if [[ "${WITH_PERF}" -eq 1 ]]; then
  echo "[status-check] running optional perf gate"
  bash tools/perf-check/run-perf-check.sh
fi

echo "[status-check] all checks passed"
