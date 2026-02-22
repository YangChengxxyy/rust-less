#!/usr/bin/env node

const fs = require("fs");
const path = require("path");

const REPO_ROOT = path.resolve(__dirname, "..", "..");

function parseArgs(argv) {
  const args = {
    baseline: path.join(REPO_ROOT, "docs", "PERF_BASELINE.json"),
    criterionDir: path.join(REPO_ROOT, "target", "criterion"),
    reportJson: path.join(REPO_ROOT, "target", "perf-check", "report.json"),
    reportMd: path.join(REPO_ROOT, "target", "perf-check", "report.md"),
    defaultThreshold: null,
    updateBaseline: false,
  };

  for (let i = 2; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--baseline") {
      args.baseline = path.resolve(argv[++i]);
    } else if (arg === "--criterion-dir") {
      args.criterionDir = path.resolve(argv[++i]);
    } else if (arg === "--report-json") {
      args.reportJson = path.resolve(argv[++i]);
    } else if (arg === "--report-md") {
      args.reportMd = path.resolve(argv[++i]);
    } else if (arg === "--default-threshold") {
      args.defaultThreshold = Number(argv[++i]);
      if (Number.isNaN(args.defaultThreshold) || args.defaultThreshold < 0) {
        throw new Error("Invalid --default-threshold value");
      }
    } else if (arg === "--update-baseline") {
      args.updateBaseline = true;
    } else if (arg === "--help" || arg === "-h") {
      printHelp();
      process.exit(0);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  return args;
}

function printHelp() {
  console.log(`Usage: node tools/perf-check/check-perf-regression.js [options]

Options:
  --baseline <path>           Baseline json path (default: docs/PERF_BASELINE.json)
  --criterion-dir <path>      Criterion output root (default: target/criterion)
  --report-json <path>        Report json output path
  --report-md <path>          Report markdown output path
  --default-threshold <pct>   Override max regression threshold percentage
  --update-baseline           Write baseline from current Criterion outputs and exit
  -h, --help                  Show help
`);
}

function ensureDir(filePath) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function writeJson(filePath, data) {
  ensureDir(filePath);
  fs.writeFileSync(filePath, JSON.stringify(data, null, 2) + "\n");
}

function readCurrentBenchmarks(criterionDir) {
  if (!fs.existsSync(criterionDir)) {
    throw new Error(`Criterion dir not found: ${criterionDir}`);
  }

  const result = {};
  const entries = fs.readdirSync(criterionDir, { withFileTypes: true });
  for (const entry of entries) {
    if (!entry.isDirectory()) {
      continue;
    }
    const estimatesPath = path.join(
      criterionDir,
      entry.name,
      "new",
      "estimates.json"
    );
    if (!fs.existsSync(estimatesPath)) {
      continue;
    }
    const estimates = readJson(estimatesPath);
    const mean = estimates && estimates.mean ? estimates.mean.point_estimate : null;
    if (typeof mean !== "number" || Number.isNaN(mean)) {
      continue;
    }
    result[entry.name] = mean;
  }

  return result;
}

function humanNs(ns) {
  if (ns >= 1_000_000) {
    return `${(ns / 1_000_000).toFixed(2)} ms`;
  }
  if (ns >= 1_000) {
    return `${(ns / 1_000).toFixed(2)} us`;
  }
  return `${ns.toFixed(2)} ns`;
}

function percent(n) {
  const sign = n > 0 ? "+" : "";
  return `${sign}${n.toFixed(2)}%`;
}

function buildMarkdown(report) {
  const lines = [];
  lines.push("# Performance Regression Report");
  lines.push("");
  lines.push(`- generated_at: ${report.generated_at}`);
  lines.push(`- baseline: ${report.baseline}`);
  lines.push(`- criterion_dir: ${report.criterion_dir}`);
  lines.push(
    `- summary: pass=${report.summary.pass}, fail=${report.summary.fail}, missing=${report.summary.missing}`
  );
  lines.push("");
  lines.push(
    "| benchmark | baseline | current | delta | threshold | status |"
  );
  lines.push(
    "|-----------|----------|---------|-------|-----------|--------|"
  );

  for (const row of report.results) {
    lines.push(
      `| ${row.benchmark} | ${humanNs(row.baseline_ns)} | ${humanNs(row.current_ns)} | ${percent(row.regression_pct)} | ${row.max_regression_pct.toFixed(2)}% | ${row.status} |`
    );
  }

  if (report.missing_benchmarks.length > 0) {
    lines.push("");
    lines.push("## Missing Benchmarks");
    for (const name of report.missing_benchmarks) {
      lines.push(`- ${name}`);
    }
  }

  if (report.extra_benchmarks.length > 0) {
    lines.push("");
    lines.push("## Extra Benchmarks");
    for (const name of report.extra_benchmarks) {
      lines.push(`- ${name}`);
    }
  }

  lines.push("");
  return lines.join("\n");
}

function updateBaseline(args) {
  const current = readCurrentBenchmarks(args.criterionDir);
  const existing = fs.existsSync(args.baseline) ? readJson(args.baseline) : null;
  const defaultThreshold =
    args.defaultThreshold ??
    (existing && typeof existing.default_max_regression_pct === "number"
      ? existing.default_max_regression_pct
      : 20);

  const thresholds = {};
  if (existing && existing.benchmarks) {
    for (const [name, data] of Object.entries(existing.benchmarks)) {
      if (data && typeof data.max_regression_pct === "number") {
        thresholds[name] = data.max_regression_pct;
      }
    }
  }

  const baseline = {
    version: 1,
    generated_at: new Date().toISOString().slice(0, 10),
    unit: "ns",
    default_max_regression_pct: defaultThreshold,
    benchmarks: {},
  };

  for (const [name, mean] of Object.entries(current)) {
    baseline.benchmarks[name] = { mean_ns: mean };
    if (typeof thresholds[name] === "number") {
      baseline.benchmarks[name].max_regression_pct = thresholds[name];
    }
  }

  writeJson(args.baseline, baseline);
  console.log(`Updated baseline: ${args.baseline}`);
  console.log(`Benchmarks captured: ${Object.keys(current).length}`);
}

function compare(args) {
  if (!fs.existsSync(args.baseline)) {
    throw new Error(`Baseline file not found: ${args.baseline}`);
  }

  const baseline = readJson(args.baseline);
  const current = readCurrentBenchmarks(args.criterionDir);
  const baselineBenchmarks = baseline.benchmarks || {};
  const defaultThreshold =
    args.defaultThreshold ??
    (typeof baseline.default_max_regression_pct === "number"
      ? baseline.default_max_regression_pct
      : 20);

  const results = [];
  const missingBenchmarks = [];
  const baselineNames = Object.keys(baselineBenchmarks).sort();

  for (const name of baselineNames) {
    const baselineEntry = baselineBenchmarks[name];
    const baselineNs =
      baselineEntry && typeof baselineEntry.mean_ns === "number"
        ? baselineEntry.mean_ns
        : null;
    if (baselineNs === null || baselineNs <= 0) {
      throw new Error(`Invalid baseline mean_ns for benchmark: ${name}`);
    }

    const currentNs = current[name];
    if (typeof currentNs !== "number") {
      missingBenchmarks.push(name);
      continue;
    }

    const regressionPct = ((currentNs - baselineNs) / baselineNs) * 100;
    const maxRegressionPct =
      baselineEntry && typeof baselineEntry.max_regression_pct === "number"
        ? baselineEntry.max_regression_pct
        : defaultThreshold;
    const status = regressionPct > maxRegressionPct ? "fail" : "pass";

    results.push({
      benchmark: name,
      baseline_ns: baselineNs,
      current_ns: currentNs,
      regression_pct: regressionPct,
      max_regression_pct: maxRegressionPct,
      status,
    });
  }

  results.sort((a, b) => a.benchmark.localeCompare(b.benchmark));

  const extraBenchmarks = Object.keys(current)
    .filter((name) => !(name in baselineBenchmarks))
    .sort();
  const failCount = results.filter((r) => r.status === "fail").length;
  const passCount = results.filter((r) => r.status === "pass").length;

  const report = {
    generated_at: new Date().toISOString(),
    baseline: path.relative(REPO_ROOT, args.baseline) || args.baseline,
    criterion_dir:
      path.relative(REPO_ROOT, args.criterionDir) || args.criterionDir,
    summary: {
      total: results.length,
      pass: passCount,
      fail: failCount,
      missing: missingBenchmarks.length,
    },
    results,
    missing_benchmarks: missingBenchmarks,
    extra_benchmarks: extraBenchmarks,
  };

  if (args.reportJson) {
    writeJson(args.reportJson, report);
    console.log(`Wrote json report: ${args.reportJson}`);
  }
  if (args.reportMd) {
    ensureDir(args.reportMd);
    fs.writeFileSync(args.reportMd, buildMarkdown(report));
    console.log(`Wrote markdown report: ${args.reportMd}`);
  }

  console.log(
    `Summary: pass=${passCount}, fail=${failCount}, missing=${missingBenchmarks.length}, extra=${extraBenchmarks.length}`
  );

  for (const row of results) {
    console.log(
      `${row.benchmark}: baseline=${humanNs(row.baseline_ns)}, current=${humanNs(row.current_ns)}, delta=${percent(row.regression_pct)}, threshold=${row.max_regression_pct.toFixed(2)}%, status=${row.status}`
    );
  }

  if (missingBenchmarks.length > 0) {
    console.error(
      `Missing benchmark outputs: ${missingBenchmarks.join(", ")}`
    );
  }

  if (failCount > 0 || missingBenchmarks.length > 0) {
    process.exit(1);
  }
}

function main() {
  try {
    const args = parseArgs(process.argv);
    if (args.updateBaseline) {
      updateBaseline(args);
      return;
    }
    compare(args);
  } catch (err) {
    console.error(`perf-check failed: ${err.message}`);
    process.exit(1);
  }
}

main();
