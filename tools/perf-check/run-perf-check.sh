#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BASELINE="${ROOT_DIR}/docs/PERF_BASELINE.json"
CRITERION_DIR="${ROOT_DIR}/target/criterion"
REPORT_JSON="${ROOT_DIR}/target/perf-check/report.json"
REPORT_MD="${ROOT_DIR}/target/perf-check/report.md"

cd "${ROOT_DIR}"

echo "[perf-check] 1/2 cargo bench --bench compiler_bench"
cargo bench --bench compiler_bench

echo "[perf-check] 2/2 compare with baseline"
node tools/perf-check/check-perf-regression.js \
  --baseline "${BASELINE}" \
  --criterion-dir "${CRITERION_DIR}" \
  --report-json "${REPORT_JSON}" \
  --report-md "${REPORT_MD}"

echo "[perf-check] all checks passed"
