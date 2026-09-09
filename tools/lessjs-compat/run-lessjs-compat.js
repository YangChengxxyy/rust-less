#!/usr/bin/env node

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const REPO_ROOT = path.resolve(__dirname, "..", "..");
const DEFAULT_MD = path.join(REPO_ROOT, "docs", "LESSJS_DIFF_REPORT.md");
const DEFAULT_JSON = path.join(REPO_ROOT, "docs", "LESSJS_DIFF_REPORT.json");

const CASES = [
  {
    id: "maps-map-merge-shallow",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-merge.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-map-deep-merge-order",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-deep-merge.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-map-key-normalization-extension",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-key-normalization.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-native-unquoted-key",
    category: "maps",
    baseline: "lessjs-native",
    file: "tests/fixtures/lessjs-compat/map-native-unquoted-key.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-native-quoted-key",
    category: "maps",
    baseline: "lessjs-native",
    file: "tests/fixtures/lessjs-compat/map-native-quoted-key.less",
    compare: "error",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-map-update-missing-error",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-update-missing.less",
    compare: "error",
    compress: false,
    sourceMap: false,
  },
  {
    id: "maps-map-replace-missing-error",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-replace-missing.less",
    compare: "error",
    compress: false,
    sourceMap: false,
  },
  {
    id: "maps-map-intermediate-not-map-error",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-intermediate-not-map.less",
    compare: "error",
    compress: false,
    sourceMap: false,
  },
  {
    id: "maps-map-deep-nested-read-write-delete",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-deep-nested-combo.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-map-deep-remove-prune-branch",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-deep-remove-prune-branch.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-map-key-normalization-write-path",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-key-normalization-write.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-map-update-deep-intermediate-error",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-update-deep-intermediate.less",
    compare: "error",
    compress: false,
    sourceMap: false,
  },
  {
    id: "maps-map-set-non-map-argument-error",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-set-non-map.less",
    compare: "error",
    compress: false,
    sourceMap: false,
  },
  {
    id: "maps-map-set-empty-path-error",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-set-empty-path.less",
    compare: "error",
    compress: false,
    sourceMap: false,
  },
  {
    id: "maps-map-update-non-map-argument-error",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-update-non-map.less",
    compare: "error",
    compress: false,
    sourceMap: false,
  },
  {
    id: "maps-map-replace-intermediate-not-map-error",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-replace-intermediate-not-map.less",
    compare: "error",
    compress: false,
    sourceMap: false,
  },
  {
    id: "maps-native-each-map",
    category: "maps",
    baseline: "lessjs-native",
    file: "tests/fixtures/lessjs-compat/map-native-each-map.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-native-each-list",
    category: "maps",
    baseline: "lessjs-native",
    file: "tests/fixtures/lessjs-compat/map-native-each-list.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-native-each-in-rule",
    category: "maps",
    baseline: "lessjs-native",
    file: "tests/fixtures/lessjs-compat/map-native-each-in-rule.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-native-map-override",
    category: "maps",
    baseline: "lessjs-native",
    file: "tests/fixtures/lessjs-compat/map-native-map-override.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-native-media-prelude",
    category: "maps",
    baseline: "lessjs-native",
    file: "tests/fixtures/lessjs-compat/map-native-media-prelude.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-native-dup-key-last-wins",
    category: "maps",
    baseline: "lessjs-native",
    file: "tests/fixtures/lessjs-compat/map-native-dup-key-last-wins.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-native-unit-negative-keys",
    category: "maps",
    baseline: "lessjs-native",
    file: "tests/fixtures/lessjs-compat/map-native-unit-negative-keys.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-native-lazy-value",
    category: "maps",
    baseline: "lessjs-native",
    file: "tests/fixtures/lessjs-compat/map-native-lazy-value.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-native-interpolated-key",
    category: "maps",
    baseline: "lessjs-native",
    file: "tests/fixtures/lessjs-compat/map-native-interpolated-key.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-native-map-as-value-error",
    category: "maps",
    baseline: "lessjs-native",
    file: "tests/fixtures/lessjs-compat/map-native-map-as-value-error.less",
    compare: "error",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-map-variable-key-access",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-variable-key-access.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "maps-map-nested-bracket-access",
    category: "maps",
    baseline: "extension",
    file: "tests/fixtures/lessjs-compat/map-nested-bracket-access.less",
    compare: "css",
    compress: true,
    sourceMap: false,
  },
  {
    id: "sourcemap-imported-keyframes",
    category: "source-map",
    file: "tests/fixtures/keyframes-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    includePaths: ["tests/fixtures"],
    expectedSources: ["keyframes-import.less"],
  },
  {
    id: "sourcemap-imported-keyframes-source-root",
    category: "source-map",
    file: "tests/fixtures/keyframes-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    includePaths: ["tests/fixtures"],
    expectedSources: ["keyframes-import.less"],
  },
  {
    id: "sourcemap-media-bubble-import",
    category: "source-map",
    file: "tests/fixtures/media-bubble-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    includePaths: ["tests/fixtures"],
    expectedSources: ["media-bubble-import.less"],
  },
  {
    id: "sourcemap-supports-bubble-import",
    category: "source-map",
    file: "tests/fixtures/supports-bubble-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    includePaths: ["tests/fixtures"],
    expectedSources: ["supports-bubble-import.less"],
  },
  {
    id: "sourcemap-local-rule",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-local-rule.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
  },
  {
    id: "sourcemap-import-same-dir",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-import-same-dir-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["sourcemap-import-same-dir-source.less"],
  },
  {
    id: "sourcemap-import-nested",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-import-nested-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["source.less"],
  },
  {
    id: "sourcemap-media-local",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-media-local.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
  },
  {
    id: "sourcemap-import-chain-nested-atrule",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-import-chain-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["level2.less"],
  },
  {
    id: "sourcemap-import-chain-nested-atrule-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-import-chain-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["level2.less"],
  },
  {
    id: "sourcemap-import-chain-keyframes",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-chain-keyframes-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["level2.less"],
  },
  {
    id: "sourcemap-import-chain-keyframes-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-chain-keyframes-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["level2.less"],
  },
  {
    id: "sourcemap-import-chain-media-bubble",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-chain-media-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["level2.less"],
  },
  {
    id: "sourcemap-import-chain-media-bubble-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-chain-media-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["level2.less"],
  },
  {
    id: "sourcemap-multi-import-cross-dir",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-multi-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["button.less", "light.less"],
  },
  {
    id: "sourcemap-multi-import-cross-dir-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-multi-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["button.less", "light.less"],
  },
  {
    id: "sourcemap-import-parent-traversal-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["value.less"],
  },
  {
    id: "sourcemap-import-parent-traversal",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["value.less"],
  },
  {
    id: "sourcemap-parent-multi-import-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-multi-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["one.less", "two.less"],
  },
  {
    id: "sourcemap-parent-multi-import",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-multi-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["one.less", "two.less"],
  },
  {
    id: "sourcemap-parent-media-bubble-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-media-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["value.less"],
  },
  {
    id: "sourcemap-parent-media-bubble",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-media-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["value.less"],
  },
  {
    id: "sourcemap-parent-deep-media-supports-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["level2.less"],
  },
  {
    id: "sourcemap-parent-deep-media-supports",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["level2.less"],
  },
  {
    id: "sourcemap-parent-deep-keyframes-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-keyframes-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["anim.less"],
  },
  {
    id: "sourcemap-parent-deep-keyframes",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-keyframes-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["anim.less"],
  },
  {
    id: "sourcemap-parent-mixed-multi-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-mixed-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["one.less", "two.less"],
  },
  {
    id: "sourcemap-parent-mixed-multi",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-mixed-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["one.less", "two.less"],
  },
  {
    id: "sourcemap-duplicate-basename-multi-dir",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate/one/shared.less",
      "sourcemap-duplicate/two/shared.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-duplicate-basename-multi-dir-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate/one/shared.less",
      "sourcemap-duplicate/two/shared.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-duplicate-parent-traversal",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-parent-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-parent/feature/sub/main.less",
      "sourcemap-duplicate-parent/shared/one/shared.less",
      "sourcemap-duplicate-parent/shared/two/shared.less",
    ],
    expectedSourceCount: 3,
  },
  {
    id: "sourcemap-duplicate-parent-traversal-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-parent-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-parent/feature/sub/main.less",
      "sourcemap-duplicate-parent/shared/one/shared.less",
      "sourcemap-duplicate-parent/shared/two/shared.less",
    ],
    expectedSourceCount: 3,
  },
  {
    id: "sourcemap-duplicate-chain",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain/feature/sub/main.less",
      "sourcemap-duplicate-chain/shared/one/entry.less",
      "sourcemap-duplicate-chain/shared/two/entry.less",
      "sourcemap-duplicate-chain/shared/one/shared.less",
      "sourcemap-duplicate-chain/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-duplicate-chain-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain/feature/sub/main.less",
      "sourcemap-duplicate-chain/shared/one/entry.less",
      "sourcemap-duplicate-chain/shared/two/entry.less",
      "sourcemap-duplicate-chain/shared/one/shared.less",
      "sourcemap-duplicate-chain/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-duplicate-chain-keyframes",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-keyframes-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-keyframes/feature/sub/main.less",
      "sourcemap-duplicate-chain-keyframes/shared/one/entry.less",
      "sourcemap-duplicate-chain-keyframes/shared/two/entry.less",
      "sourcemap-duplicate-chain-keyframes/shared/one/shared.less",
      "sourcemap-duplicate-chain-keyframes/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-duplicate-chain-keyframes-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-keyframes-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-keyframes/feature/sub/main.less",
      "sourcemap-duplicate-chain-keyframes/shared/one/entry.less",
      "sourcemap-duplicate-chain-keyframes/shared/two/entry.less",
      "sourcemap-duplicate-chain-keyframes/shared/one/shared.less",
      "sourcemap-duplicate-chain-keyframes/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-duplicate-chain-supports",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-supports-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-supports/feature/sub/main.less",
      "sourcemap-duplicate-chain-supports/shared/one/entry.less",
      "sourcemap-duplicate-chain-supports/shared/two/entry.less",
      "sourcemap-duplicate-chain-supports/shared/one/shared.less",
      "sourcemap-duplicate-chain-supports/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-duplicate-chain-supports-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-supports-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-supports/feature/sub/main.less",
      "sourcemap-duplicate-chain-supports/shared/one/entry.less",
      "sourcemap-duplicate-chain-supports/shared/two/entry.less",
      "sourcemap-duplicate-chain-supports/shared/one/shared.less",
      "sourcemap-duplicate-chain-supports/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-duplicate-chain-media",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media/feature/sub/main.less",
      "sourcemap-duplicate-chain-media/shared/one/entry.less",
      "sourcemap-duplicate-chain-media/shared/two/entry.less",
      "sourcemap-duplicate-chain-media/shared/one/shared.less",
      "sourcemap-duplicate-chain-media/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-duplicate-chain-media-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media/feature/sub/main.less",
      "sourcemap-duplicate-chain-media/shared/one/entry.less",
      "sourcemap-duplicate-chain-media/shared/two/entry.less",
      "sourcemap-duplicate-chain-media/shared/one/shared.less",
      "sourcemap-duplicate-chain-media/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-duplicate-chain-media-and",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-and-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media-and/feature/sub/main.less",
      "sourcemap-duplicate-chain-media-and/shared/one/entry.less",
      "sourcemap-duplicate-chain-media-and/shared/two/entry.less",
      "sourcemap-duplicate-chain-media-and/shared/one/shared.less",
      "sourcemap-duplicate-chain-media-and/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-duplicate-chain-media-and-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-and-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media-and/feature/sub/main.less",
      "sourcemap-duplicate-chain-media-and/shared/one/entry.less",
      "sourcemap-duplicate-chain-media-and/shared/two/entry.less",
      "sourcemap-duplicate-chain-media-and/shared/one/shared.less",
      "sourcemap-duplicate-chain-media-and/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-parent-deep-media-and",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-media-and-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "level2.less"],
    expectedSourceSuffixes: [
      "sourcemap-parent-deep-media-and/feature/level1/main.less",
      "sourcemap-parent-deep-media-and/shared/level2.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-parent-deep-media-and-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-media-and-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "level2.less"],
    expectedSourceSuffixes: [
      "sourcemap-parent-deep-media-and/feature/level1/main.less",
      "sourcemap-parent-deep-media-and/shared/level2.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-duplicate-chain-media-only-not",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-only-not-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media-only-not/feature/sub/main.less",
      "sourcemap-duplicate-chain-media-only-not/shared/one/entry.less",
      "sourcemap-duplicate-chain-media-only-not/shared/two/entry.less",
      "sourcemap-duplicate-chain-media-only-not/shared/one/shared.less",
      "sourcemap-duplicate-chain-media-only-not/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-duplicate-chain-media-only-not-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-only-not-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media-only-not/feature/sub/main.less",
      "sourcemap-duplicate-chain-media-only-not/shared/one/entry.less",
      "sourcemap-duplicate-chain-media-only-not/shared/two/entry.less",
      "sourcemap-duplicate-chain-media-only-not/shared/one/shared.less",
      "sourcemap-duplicate-chain-media-only-not/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-parent-deep-media-comma-not",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-media-comma-not-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "level2.less"],
    expectedSourceSuffixes: [
      "sourcemap-parent-deep-media-comma-not/feature/level1/main.less",
      "sourcemap-parent-deep-media-comma-not/shared/level2.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-parent-deep-media-comma-not-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-media-comma-not-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "level2.less"],
    expectedSourceSuffixes: [
      "sourcemap-parent-deep-media-comma-not/feature/level1/main.less",
      "sourcemap-parent-deep-media-comma-not/shared/level2.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-duplicate-chain-media-not-only-supports",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-not-only-supports-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media-not-only-supports/feature/sub/main.less",
      "sourcemap-duplicate-chain-media-not-only-supports/shared/one/entry.less",
      "sourcemap-duplicate-chain-media-not-only-supports/shared/two/entry.less",
      "sourcemap-duplicate-chain-media-not-only-supports/shared/one/shared.less",
      "sourcemap-duplicate-chain-media-not-only-supports/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-duplicate-chain-media-not-only-supports-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-not-only-supports-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media-not-only-supports/feature/sub/main.less",
      "sourcemap-duplicate-chain-media-not-only-supports/shared/one/entry.less",
      "sourcemap-duplicate-chain-media-not-only-supports/shared/two/entry.less",
      "sourcemap-duplicate-chain-media-not-only-supports/shared/one/shared.less",
      "sourcemap-duplicate-chain-media-not-only-supports/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-parent-deep-media-print-not-only",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-media-print-not-only-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "level2.less"],
    expectedSourceSuffixes: [
      "sourcemap-parent-deep-media-print-not-only/feature/level1/main.less",
      "sourcemap-parent-deep-media-print-not-only/shared/level2.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-parent-deep-media-print-not-only-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-media-print-not-only-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "level2.less"],
    expectedSourceSuffixes: [
      "sourcemap-parent-deep-media-print-not-only/feature/level1/main.less",
      "sourcemap-parent-deep-media-print-not-only/shared/level2.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-duplicate-chain-media-calc",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-calc-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media-calc/feature/sub/main.less",
      "sourcemap-duplicate-chain-media-calc/shared/one/entry.less",
      "sourcemap-duplicate-chain-media-calc/shared/two/entry.less",
      "sourcemap-duplicate-chain-media-calc/shared/one/shared.less",
      "sourcemap-duplicate-chain-media-calc/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-duplicate-chain-media-calc-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-calc-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media-calc/feature/sub/main.less",
      "sourcemap-duplicate-chain-media-calc/shared/one/entry.less",
      "sourcemap-duplicate-chain-media-calc/shared/two/entry.less",
      "sourcemap-duplicate-chain-media-calc/shared/one/shared.less",
      "sourcemap-duplicate-chain-media-calc/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-parent-deep-media-calc",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-media-calc-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "level2.less"],
    expectedSourceSuffixes: [
      "sourcemap-parent-deep-media-calc/feature/level1/main.less",
      "sourcemap-parent-deep-media-calc/shared/level2.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-parent-deep-media-calc-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-media-calc-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "level2.less"],
    expectedSourceSuffixes: [
      "sourcemap-parent-deep-media-calc/feature/level1/main.less",
      "sourcemap-parent-deep-media-calc/shared/level2.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-duplicate-chain-media-env-var",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-env-var-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media-env-var/feature/sub/main.less",
      "sourcemap-duplicate-chain-media-env-var/shared/one/entry.less",
      "sourcemap-duplicate-chain-media-env-var/shared/two/entry.less",
      "sourcemap-duplicate-chain-media-env-var/shared/one/shared.less",
      "sourcemap-duplicate-chain-media-env-var/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-duplicate-chain-media-env-var-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-env-var-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media-env-var/feature/sub/main.less",
      "sourcemap-duplicate-chain-media-env-var/shared/one/entry.less",
      "sourcemap-duplicate-chain-media-env-var/shared/two/entry.less",
      "sourcemap-duplicate-chain-media-env-var/shared/one/shared.less",
      "sourcemap-duplicate-chain-media-env-var/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-parent-deep-media-func-mix",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-media-func-mix-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "level2.less"],
    expectedSourceSuffixes: [
      "sourcemap-parent-deep-media-func-mix/feature/level1/main.less",
      "sourcemap-parent-deep-media-func-mix/shared/level2.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-parent-deep-media-func-mix-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-media-func-mix-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "level2.less"],
    expectedSourceSuffixes: [
      "sourcemap-parent-deep-media-func-mix/feature/level1/main.less",
      "sourcemap-parent-deep-media-func-mix/shared/level2.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-duplicate-chain-media-url-var",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-url-var-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media-url-var/feature/sub/main.less",
      "sourcemap-duplicate-chain-media-url-var/shared/one/entry.less",
      "sourcemap-duplicate-chain-media-url-var/shared/two/entry.less",
      "sourcemap-duplicate-chain-media-url-var/shared/one/shared.less",
      "sourcemap-duplicate-chain-media-url-var/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-duplicate-chain-media-url-var-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-url-var-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media-url-var/feature/sub/main.less",
      "sourcemap-duplicate-chain-media-url-var/shared/one/entry.less",
      "sourcemap-duplicate-chain-media-url-var/shared/two/entry.less",
      "sourcemap-duplicate-chain-media-url-var/shared/one/shared.less",
      "sourcemap-duplicate-chain-media-url-var/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-parent-deep-media-url-var",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-media-url-var-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "level2.less"],
    expectedSourceSuffixes: [
      "sourcemap-parent-deep-media-url-var/feature/level1/main.less",
      "sourcemap-parent-deep-media-url-var/shared/level2.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-parent-deep-media-url-var-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-media-url-var-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "level2.less"],
    expectedSourceSuffixes: [
      "sourcemap-parent-deep-media-url-var/feature/level1/main.less",
      "sourcemap-parent-deep-media-url-var/shared/level2.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-duplicate-chain-media-orientation-resolution",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-orientation-resolution-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media-orientation-resolution/feature/sub/main.less",
      "sourcemap-duplicate-chain-media-orientation-resolution/shared/one/entry.less",
      "sourcemap-duplicate-chain-media-orientation-resolution/shared/two/entry.less",
      "sourcemap-duplicate-chain-media-orientation-resolution/shared/one/shared.less",
      "sourcemap-duplicate-chain-media-orientation-resolution/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-duplicate-chain-media-orientation-resolution-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-duplicate-chain-media-orientation-resolution-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "entry.less", "shared.less"],
    expectedSourceSuffixes: [
      "sourcemap-duplicate-chain-media-orientation-resolution/feature/sub/main.less",
      "sourcemap-duplicate-chain-media-orientation-resolution/shared/one/entry.less",
      "sourcemap-duplicate-chain-media-orientation-resolution/shared/two/entry.less",
      "sourcemap-duplicate-chain-media-orientation-resolution/shared/one/shared.less",
      "sourcemap-duplicate-chain-media-orientation-resolution/shared/two/shared.less",
    ],
    expectedSourceCount: 5,
  },
  {
    id: "sourcemap-parent-deep-media-custom-func",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-media-custom-func-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["main.less", "level2.less"],
    expectedSourceSuffixes: [
      "sourcemap-parent-deep-media-custom-func/feature/level1/main.less",
      "sourcemap-parent-deep-media-custom-func/shared/level2.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-parent-deep-media-custom-func-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-parent-deep-media-custom-func-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["main.less", "level2.less"],
    expectedSourceSuffixes: [
      "sourcemap-parent-deep-media-custom-func/feature/level1/main.less",
      "sourcemap-parent-deep-media-custom-func/shared/level2.less",
    ],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-triple-chain-mixed-atrule",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-triple-chain-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["level1.less", "level2.less", "level3.less"],
    expectedSourceCount: 4,
  },
  {
    id: "sourcemap-triple-chain-mixed-atrule-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-triple-chain-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["level1.less", "level2.less", "level3.less"],
    expectedSourceCount: 4,
  },
  {
    id: "sourcemap-detached-ruleset-import",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-detached-ruleset-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    expectedSources: ["lib.less"],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-detached-ruleset-import-source-root",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-detached-ruleset-entry.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
    sourceMapRoot: "/workspace/src",
    expectedSources: ["lib.less"],
    expectedSourceCount: 2,
  },
  {
    id: "sourcemap-media-prelude-var",
    category: "source-map",
    file: "tests/fixtures/lessjs-compat/sourcemap-media-prelude-var.less",
    compare: "css",
    compress: false,
    sourceMap: true,
    sourceMapLessjsCompat: true,
  },
];

function parseArgs(argv) {
  const args = {
    outputMd: DEFAULT_MD,
    outputJson: DEFAULT_JSON,
    caseFilter: null,
    strict: false,
    strictMappings: false,
    observeMappings: false,
  };

  for (let i = 2; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--output-md") {
      args.outputMd = path.resolve(argv[++i]);
    } else if (arg === "--output-json") {
      args.outputJson = path.resolve(argv[++i]);
    } else if (arg === "--case") {
      args.caseFilter = argv[++i];
    } else if (arg === "--strict") {
      args.strict = true;
    } else if (arg === "--strict-mappings") {
      args.strictMappings = true;
    } else if (arg === "--observe-mappings") {
      args.observeMappings = true;
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
  console.log(`Usage: node tools/lessjs-compat/run-lessjs-compat.js [options]

Options:
  --output-md <path>    Markdown report output path (default: docs/LESSJS_DIFF_REPORT.md)
  --output-json <path>  JSON report output path (default: docs/LESSJS_DIFF_REPORT.json)
  --case <pattern>      Run only cases whose id includes <pattern>
  --strict              Exit with non-zero code on fail/blocked cases
  --strict-mappings     Treat source map mappings hash mismatch as fail
  --observe-mappings    Observe source map mappings hash mismatch (non-fail)
  -h, --help            Show help
`);
}

function runCommand(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd || REPO_ROOT,
    encoding: "utf8",
  });

  return {
    status: result.status,
    stdout: (result.stdout || "").trim(),
    stderr: (result.stderr || "").trim(),
    error: result.error,
  };
}

function ensureDir(filePath) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
}

function ensureRustLessBinary() {
  const envBin = process.env.RUST_LESS_BIN;
  if (envBin && fs.existsSync(envBin)) {
    return path.resolve(envBin);
  }

  const candidate = path.join(REPO_ROOT, "target", "debug", "rust-less");
  if (fs.existsSync(candidate)) {
    return candidate;
  }

  const build = runCommand("cargo", ["build", "--features", "cli", "--bin", "rust-less"]);
  if (build.status !== 0) {
    const detail = build.stderr || build.stdout || "unknown build error";
    throw new Error(`Failed to build rust-less CLI: ${detail}`);
  }

  if (!fs.existsSync(candidate)) {
    throw new Error(`Built CLI not found at expected path: ${candidate}`);
  }

  return candidate;
}

function loadLessModule() {
  try {
    // eslint-disable-next-line global-require, import/no-dynamic-require
    return require("less");
  } catch (_err) {
    return null;
  }
}

function createTempRoot() {
  const tempRoot = path.join(REPO_ROOT, "target", "lessjs-compat");
  fs.rmSync(tempRoot, { recursive: true, force: true });
  fs.mkdirSync(tempRoot, { recursive: true });
  return tempRoot;
}

function resolveCase(caseDef) {
  const inputPath = path.join(REPO_ROOT, caseDef.file);
  const includePaths = [path.dirname(inputPath)];
  if (Array.isArray(caseDef.includePaths)) {
    for (const rel of caseDef.includePaths) {
      includePaths.push(path.join(REPO_ROOT, rel));
    }
  }

  return {
    ...caseDef,
    baseline: caseDef.baseline || "lessjs-native",
    inputPath,
    includePaths: Array.from(new Set(includePaths)),
  };
}

function compileWithRust(caseDef, rustBin, tempRoot) {
  const caseDir = path.join(tempRoot, caseDef.id, "rust");
  fs.mkdirSync(caseDir, { recursive: true });

  const cssBaseName = `${path.basename(caseDef.inputPath, path.extname(caseDef.inputPath))}.css`;
  const outputCssPath = path.join(caseDir, cssBaseName);
  const outputMapPath = path.join(caseDir, `${cssBaseName}.map`);

  const args = [caseDef.inputPath, "-o", outputCssPath];
  if (caseDef.compress) {
    args.push("--compress");
  }
  if (caseDef.sourceMap) {
    args.push("--source-map", "--source-map-file", outputMapPath);
    if (caseDef.sourceMapRoot) {
      args.push("--source-map-root", caseDef.sourceMapRoot);
    }
    if (caseDef.sourceMapLessjsCompat) {
      args.push("--source-map-lessjs-compat");
    }
  }
  for (const includePath of caseDef.includePaths) {
    args.push("--include-path", includePath);
  }

  const result = runCommand(rustBin, args, { cwd: REPO_ROOT });
  if (result.status !== 0) {
    return {
      ok: false,
      error: result.stderr || result.stdout || "rust-less failed",
      command: `${rustBin} ${args.join(" ")}`,
    };
  }

  const css = fs.readFileSync(outputCssPath, "utf8");
  const map = caseDef.sourceMap && fs.existsSync(outputMapPath)
    ? fs.readFileSync(outputMapPath, "utf8")
    : null;

  return {
    ok: true,
    css,
    map,
    command: `${rustBin} ${args.join(" ")}`,
  };
}

async function compileWithLessJs(caseDef, less) {
  if (!less) {
    return {
      ok: false,
      blocked: true,
      error: "less module missing (run: npm install --prefix tools/lessjs-compat)",
    };
  }

  const source = fs.readFileSync(caseDef.inputPath, "utf8");
  const options = {
    filename: caseDef.inputPath,
    paths: caseDef.includePaths,
    compress: Boolean(caseDef.compress),
  };

  if (caseDef.sourceMap) {
    options.sourceMap = {};
    if (caseDef.sourceMapRoot) {
      options.sourceMap.sourceMapRootpath = caseDef.sourceMapRoot;
    }
  }

  try {
    const rendered = await less.render(source, options);
    return {
      ok: true,
      css: rendered.css,
      map: rendered.map || null,
    };
  } catch (err) {
    const line = err.line ? ` line ${err.line}` : "";
    const column = err.column ? ` col ${err.column}` : "";
    const filename = err.filename ? ` (${path.basename(err.filename)})` : "";
    return {
      ok: false,
      error: `${err.type || "LessError"}: ${err.message}${filename}${line}${column}`,
    };
  }
}

function normalizeCss(css) {
  return css
    .replace(/\/\*# sourceMappingURL=.*?\*\//g, "")
    .replace(/\s+/g, " ")
    .replace(/\s*([{}:;,>+~])\s*/g, "$1")
    .replace(/;}/g, "}")
    .trim();
}

function diffSnippet(a, b) {
  const max = Math.min(a.length, b.length);
  let idx = 0;
  while (idx < max && a[idx] === b[idx]) {
    idx += 1;
  }

  const start = Math.max(0, idx - 40);
  const endA = Math.min(a.length, idx + 120);
  const endB = Math.min(b.length, idx + 120);

  return {
    index: idx,
    rustSnippet: a.slice(start, endA),
    lessSnippet: b.slice(start, endB),
  };
}

function parseSourceMap(rawMap) {
  if (!rawMap) {
    return { ok: false, error: "missing source map content" };
  }

  try {
    const map = JSON.parse(rawMap);
    return { ok: true, map };
  } catch (err) {
    return { ok: false, error: `invalid source map JSON: ${err.message}` };
  }
}

function normalizeStringArray(value) {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.filter((item) => typeof item === "string");
}

function hasSourceFile(sources, expectedBaseName) {
  return sources.some((sourcePath) => path.basename(sourcePath) === expectedBaseName);
}

function normalizePathSeparators(value) {
  return String(value).replace(/\\/g, "/");
}

function hasSourceSuffix(sources, expectedSuffix) {
  const normalizedSuffix = normalizePathSeparators(expectedSuffix).replace(/^\/+/, "");
  return sources.some((sourcePath) => {
    const normalizedPath = normalizePathSeparators(sourcePath);
    return (
      normalizedPath === normalizedSuffix ||
      normalizedPath.endsWith(`/${normalizedSuffix}`)
    );
  });
}

function hashString(input) {
  let hash = 2166136261;
  for (let i = 0; i < input.length; i += 1) {
    hash ^= input.charCodeAt(i);
    hash = (hash * 16777619) >>> 0;
  }
  return hash.toString(16).padStart(8, "0");
}

function summarizeSourceMap(rawMap) {
  const sources = normalizeStringArray(rawMap.sources);
  const names = normalizeStringArray(rawMap.names);
  const mappings = typeof rawMap.mappings === "string" ? rawMap.mappings : "";
  const file = typeof rawMap.file === "string" ? rawMap.file : null;
  const sourceRoot = typeof rawMap.sourceRoot === "string" ? rawMap.sourceRoot : null;

  return {
    sources,
    sourceBaseNames: sources.map((item) => path.basename(item)).sort(),
    names,
    namesCount: names.length,
    file,
    fileBaseName: file ? path.basename(file) : null,
    sourceRoot,
    mappingsLength: mappings.length,
    mappingsHash: hashString(mappings),
  };
}

function analyzeSourceMap(caseDef, rustRawMap, lessRawMap, options = {}) {
  const rust = summarizeSourceMap(rustRawMap);
  const less = summarizeSourceMap(lessRawMap);
  const observations = [];
  const structuralObservations = [];
  const encodingObservations = [];
  const strictMappings = options.strictMappings === true;
  const observeMappingsHash =
    strictMappings ||
    options.observeMappings === true ||
    (caseDef.observeMappingsHash !== false && !caseDef.sourceMapLessjsCompat);

  if (
    rust.sourceBaseNames.length !== less.sourceBaseNames.length ||
    rust.sourceBaseNames.some((name, idx) => name !== less.sourceBaseNames[idx])
  ) {
    return {
      ok: false,
      reason: "source map source files mismatch",
      details: {
        rustSources: rust.sources,
        lessSources: less.sources,
      },
    };
  }

  if (Array.isArray(caseDef.expectedSources) && caseDef.expectedSources.length > 0) {
    const missingInRust = [];
    const missingInLess = [];

    for (const expected of caseDef.expectedSources) {
      if (!hasSourceFile(rust.sources, expected)) {
        missingInRust.push(expected);
      }
      if (!hasSourceFile(less.sources, expected)) {
        missingInLess.push(expected);
      }
    }

    if (missingInRust.length > 0 || missingInLess.length > 0) {
      return {
        ok: false,
        reason: "expected source files missing in source map",
        details: {
          missingInRust,
          missingInLess,
          rustSources: rust.sources,
          lessSources: less.sources,
        },
      };
    }
  }

  if (typeof caseDef.expectedSourceCount === "number") {
    if (rust.sources.length !== caseDef.expectedSourceCount || less.sources.length !== caseDef.expectedSourceCount) {
      return {
        ok: false,
        reason: "source map source count mismatch",
        details: {
          expectedSourceCount: caseDef.expectedSourceCount,
          rustSourceCount: rust.sources.length,
          lessSourceCount: less.sources.length,
          rustSources: rust.sources,
          lessSources: less.sources,
        },
      };
    }
  }

  if (Array.isArray(caseDef.expectedSourceSuffixes) && caseDef.expectedSourceSuffixes.length > 0) {
    const missingSuffixesInRust = [];
    const missingSuffixesInLess = [];

    for (const expectedSuffix of caseDef.expectedSourceSuffixes) {
      if (!hasSourceSuffix(rust.sources, expectedSuffix)) {
        missingSuffixesInRust.push(expectedSuffix);
      }
      if (!hasSourceSuffix(less.sources, expectedSuffix)) {
        missingSuffixesInLess.push(expectedSuffix);
      }
    }

    if (missingSuffixesInRust.length > 0 || missingSuffixesInLess.length > 0) {
      return {
        ok: false,
        reason: "expected source suffixes missing in source map",
        details: {
          missingSuffixesInRust,
          missingSuffixesInLess,
          rustSources: rust.sources,
          lessSources: less.sources,
        },
      };
    }
  }

  if (rust.namesCount !== less.namesCount) {
    const message = `names count differs (rust=${rust.namesCount}, less.js=${less.namesCount})`;
    observations.push(message);
    structuralObservations.push(message);
  }

  if (rust.sourceRoot !== less.sourceRoot) {
    const message = `sourceRoot differs (rust=${rust.sourceRoot || "<none>"}, less.js=${less.sourceRoot || "<none>"})`;
    observations.push(message);
    structuralObservations.push(message);
  }

  if (rust.fileBaseName !== less.fileBaseName) {
    const message = `file basename differs (rust=${rust.fileBaseName || "<none>"}, less.js=${less.fileBaseName || "<none>"})`;
    observations.push(message);
    structuralObservations.push(message);
  }

  if (observeMappingsHash && rust.mappingsHash !== less.mappingsHash) {
    const message = `mappings hash differs (rust=${rust.mappingsHash}, less.js=${less.mappingsHash})`;
    if (strictMappings) {
      return {
        ok: false,
        reason: "source map mappings hash mismatch",
        details: {
          rust,
          less,
          strictMappings,
          observations: [message],
          structuralObservations,
          encodingObservations: [message],
        },
      };
    }
    observations.push(message);
    encodingObservations.push(message);
  }

  return {
    ok: true,
    details: {
      rust,
      less,
      strictMappings,
      observations,
      structuralObservations,
      encodingObservations,
    },
  };
}

function compareCase(caseDef, rustResult, lessResult, options = {}) {
  if (lessResult.blocked) {
    return {
      status: "blocked",
      reason: lessResult.error,
      rustOk: rustResult.ok,
      lessOk: false,
      details: {
        rustError: rustResult.error || null,
        lessError: lessResult.error,
      },
    };
  }

  if (caseDef.baseline === "extension") {
    const rustExpectedOk = caseDef.compare !== "error";
    if (rustExpectedOk && !rustResult.ok) {
      return {
        status: "fail",
        reason: "rust-less extension case failed unexpectedly",
        rustOk: false,
        lessOk: lessResult.ok,
        details: {
          rustError: rustResult.error || null,
          lessError: lessResult.error || null,
        },
      };
    }

    if (!rustExpectedOk && rustResult.ok) {
      return {
        status: "fail",
        reason: "rust-less extension case expected an error but succeeded",
        rustOk: true,
        lessOk: lessResult.ok,
        details: {
          rustError: null,
          lessError: lessResult.error || null,
        },
      };
    }

    return {
      status: "unsupported",
      reason: "less.js does not natively support rust-less map extension functions",
      rustOk: rustResult.ok,
      lessOk: lessResult.ok,
      details: {
        rustError: rustResult.error || null,
        lessError: lessResult.error || null,
      },
    };
  }

  if (caseDef.compare === "error") {
    if (!rustResult.ok && !lessResult.ok) {
      return {
        status: "pass",
        reason: "both compilers returned errors",
        rustOk: false,
        lessOk: false,
        details: {
          rustError: rustResult.error,
          lessError: lessResult.error,
        },
      };
    }

    return {
      status: "fail",
      reason: "error expectation mismatch",
      rustOk: rustResult.ok,
      lessOk: lessResult.ok,
      details: {
        rustError: rustResult.error || null,
        lessError: lessResult.error || null,
      },
    };
  }

  if (!rustResult.ok || !lessResult.ok) {
    return {
      status: "fail",
      reason: "one compiler failed while the other succeeded",
      rustOk: rustResult.ok,
      lessOk: lessResult.ok,
      details: {
        rustError: rustResult.error || null,
        lessError: lessResult.error || null,
      },
    };
  }

  const rustCss = normalizeCss(rustResult.css || "");
  const lessCss = normalizeCss(lessResult.css || "");

  if (rustCss !== lessCss) {
    return {
      status: "fail",
      reason: "compiled CSS mismatch",
      rustOk: true,
      lessOk: true,
      details: {
        diff: diffSnippet(rustCss, lessCss),
      },
    };
  }

  let passDetails = null;
  if (caseDef.sourceMap) {
    const rustMap = parseSourceMap(rustResult.map);
    const lessMap = parseSourceMap(lessResult.map);

    if (!rustMap.ok || !lessMap.ok) {
      return {
        status: "fail",
        reason: "source map generation mismatch",
        rustOk: true,
        lessOk: true,
        details: {
          rustMapError: rustMap.error || null,
          lessMapError: lessMap.error || null,
        },
      };
    }

    const sourceMapAnalysis = analyzeSourceMap(caseDef, rustMap.map, lessMap.map, {
      strictMappings: options.strictMappings,
      observeMappings: options.observeMappings,
    });
    if (!sourceMapAnalysis.ok) {
      return {
        status: "fail",
        reason: sourceMapAnalysis.reason,
        rustOk: true,
        lessOk: true,
        details: sourceMapAnalysis.details,
      };
    }

    passDetails = {
      sourceMap: sourceMapAnalysis.details,
    };
  }

  return {
    status: "pass",
    reason: "output matched",
    rustOk: rustResult.ok,
    lessOk: lessResult.ok,
    details: passDetails,
  };
}

function buildMarkdown(report) {
  const lines = [];
  lines.push("# less.js 实编译对照差异报告");
  lines.push("");
  lines.push(`- 生成时间: ${report.generatedAt}`);
  lines.push(`- Node.js: ${report.environment.nodeVersion}`);
  lines.push(`- less.js: ${report.environment.lessVersion || "missing"}`);
  lines.push(`- rust-less CLI: ${report.environment.rustLessBin}`);
  lines.push(
    `- 选项: strict=${report.environment.strict ? "on" : "off"}, strictMappings=${report.environment.strictMappings ? "on" : "off"}, observeMappings=${report.environment.observeMappings ? "on" : "off"}`,
  );
  lines.push(`- 总用例: ${report.summary.total}`);
  lines.push(`- 通过: ${report.summary.pass}`);
  lines.push(`- 观测差异（非失败）: ${report.summary.observed || 0}`);
  lines.push(`- 观测差异（结构）: ${report.summary.observedStructural || 0}`);
  lines.push(`- 观测差异（编码）: ${report.summary.observedEncoding || 0}`);
  lines.push(`- 失败: ${report.summary.fail}`);
  lines.push(`- 不支持（扩展语义）: ${report.summary.unsupported}`);
  lines.push(`- 阻塞: ${report.summary.blocked}`);
  lines.push("");

  lines.push("## 用例结果");
  lines.push("");
  lines.push("| case | category | baseline | status | reason |");
  lines.push("|---|---|---|---|---|");
  for (const item of report.results) {
    lines.push(`| ${item.id} | ${item.category} | ${item.baseline} | ${item.status} | ${item.reason} |`);
  }
  lines.push("");

  const details = report.results.filter((item) => item.status !== "pass");
  lines.push("## 失败/阻塞/不支持详情");
  lines.push("");
  if (details.length === 0) {
    lines.push("无。");
    lines.push("");
  } else {
    for (const item of details) {
      lines.push(`### ${item.id}`);
      lines.push("");
      lines.push(`- 状态: ${item.status}`);
      lines.push(`- 原因: ${item.reason}`);
      if (item.rust && item.rust.command) {
        lines.push(`- rust 命令: \`${item.rust.command}\``);
      }
      if (item.rust && item.rust.error) {
        lines.push(`- rust 错误: ${item.rust.error.replace(/\n/g, " ")}`);
      }
      if (item.less && item.less.error) {
        lines.push(`- less.js 错误: ${item.less.error.replace(/\n/g, " ")}`);
      }
      if (item.compare && item.compare.details && item.compare.details.diff) {
        lines.push(`- 首个差异索引: ${item.compare.details.diff.index}`);
        lines.push(`- rust 片段: \`${item.compare.details.diff.rustSnippet}\``);
        lines.push(`- less 片段: \`${item.compare.details.diff.lessSnippet}\``);
      }
      lines.push("");
    }
  }

  const observed = report.results.filter(
    (item) =>
      item.status === "pass" &&
      item.compare &&
      item.compare.details &&
      item.compare.details.sourceMap &&
      Array.isArray(item.compare.details.sourceMap.observations) &&
      item.compare.details.sourceMap.observations.length > 0,
  );

  lines.push("## Source Map 观测差异（非失败）");
  lines.push("");
  if (observed.length === 0) {
    lines.push("无。");
    lines.push("");
  } else {
    for (const item of observed) {
      const sourceMap = item.compare.details.sourceMap;
      lines.push(`### ${item.id}`);
      lines.push("");
      if (
        Array.isArray(sourceMap.structuralObservations) &&
        sourceMap.structuralObservations.length > 0
      ) {
        lines.push("- 结构差异:");
        for (const observation of sourceMap.structuralObservations) {
          lines.push(`  - ${observation}`);
        }
      }
      if (
        Array.isArray(sourceMap.encodingObservations) &&
        sourceMap.encodingObservations.length > 0
      ) {
        lines.push("- 编码差异:");
        for (const observation of sourceMap.encodingObservations) {
          lines.push(`  - ${observation}`);
        }
      }
      if (
        (!Array.isArray(sourceMap.structuralObservations) ||
          sourceMap.structuralObservations.length === 0) &&
        (!Array.isArray(sourceMap.encodingObservations) ||
          sourceMap.encodingObservations.length === 0)
      ) {
        for (const observation of sourceMap.observations) {
          lines.push(`- ${observation}`);
        }
      }
      lines.push(
        `- rust sources: ${sourceMap.rust.sourceBaseNames.length > 0 ? sourceMap.rust.sourceBaseNames.join(", ") : "<none>"}`,
      );
      lines.push(
        `- less.js sources: ${sourceMap.less.sourceBaseNames.length > 0 ? sourceMap.less.sourceBaseNames.join(", ") : "<none>"}`,
      );
      lines.push("");
    }
  }

  lines.push("## 结论");
  lines.push("");
  if (report.summary.blocked > 0) {
    lines.push("当前环境未满足 less.js 执行条件（缺少 npm less 依赖），已完成脚本接入，待安装依赖后可直接复跑。\n");
  } else if (report.summary.fail > 0) {
    lines.push("已发现与 less.js 的语义差异，详见上文失败项。\n");
  } else if (report.summary.unsupported > 0 && (report.summary.observed || 0) > 0) {
    lines.push(
      "less.js 原生可比场景未发现失败；扩展语义场景（map-*）为 less.js 不支持，另有 source map 观测差异（结构/编码，非失败）可用于后续对齐。\n",
    );
  } else if (report.summary.unsupported > 0) {
    lines.push("less.js 原生可比场景未发现失败；存在扩展语义场景（map-*）为 less.js 不支持，已标记 unsupported。\n");
  } else if ((report.summary.observed || 0) > 0) {
    lines.push("less.js 原生可比场景已通过；存在 source map 观测差异（结构/编码，非失败），可按需求继续收敛。\n");
  } else {
    lines.push("本批次用例与 less.js 实编译结果一致。\n");
  }

  return lines.join("\n");
}

async function main() {
  const args = parseArgs(process.argv);
  const less = loadLessModule();
  const tempRoot = createTempRoot();
  const rustBin = ensureRustLessBinary();

  const selectedCases = CASES
    .map(resolveCase)
    .filter((c) => (args.caseFilter ? c.id.includes(args.caseFilter) : true));

  if (selectedCases.length === 0) {
    throw new Error("No cases selected.");
  }

  const results = [];
  for (const caseDef of selectedCases) {
    const rustResult = compileWithRust(caseDef, rustBin, tempRoot);
    const lessResult = await compileWithLessJs(caseDef, less);
    const compare = compareCase(caseDef, rustResult, lessResult, {
      strictMappings: args.strictMappings,
      observeMappings: args.observeMappings,
    });

    results.push({
      id: caseDef.id,
      category: caseDef.category,
      baseline: caseDef.baseline,
      status: compare.status,
      reason: compare.reason,
      rust: rustResult,
      less: lessResult,
      compare,
    });
  }

  const summary = {
    total: results.length,
    pass: results.filter((r) => r.status === "pass").length,
    observed: results.filter(
      (r) =>
        r.status === "pass" &&
        r.compare &&
        r.compare.details &&
        r.compare.details.sourceMap &&
        Array.isArray(r.compare.details.sourceMap.observations) &&
        r.compare.details.sourceMap.observations.length > 0,
    ).length,
    observedStructural: results.filter((r) => {
      const sourceMap =
        r.status === "pass" &&
        r.compare &&
        r.compare.details &&
        r.compare.details.sourceMap
          ? r.compare.details.sourceMap
          : null;
      return (
        sourceMap &&
        Array.isArray(sourceMap.structuralObservations) &&
        sourceMap.structuralObservations.length > 0
      );
    }).length,
    observedEncoding: results.filter((r) => {
      const sourceMap =
        r.status === "pass" &&
        r.compare &&
        r.compare.details &&
        r.compare.details.sourceMap
          ? r.compare.details.sourceMap
          : null;
      return (
        sourceMap &&
        Array.isArray(sourceMap.encodingObservations) &&
        sourceMap.encodingObservations.length > 0
      );
    }).length,
    fail: results.filter((r) => r.status === "fail").length,
    unsupported: results.filter((r) => r.status === "unsupported").length,
    blocked: results.filter((r) => r.status === "blocked").length,
  };

  const report = {
    generatedAt: new Date().toISOString(),
    environment: {
      nodeVersion: process.version,
      lessVersion: less && Array.isArray(less.version) ? less.version.join(".") : null,
      rustLessBin: rustBin,
      tempRoot,
      strict: args.strict,
      strictMappings: args.strictMappings,
      observeMappings: args.observeMappings,
    },
    summary,
    results,
  };

  ensureDir(args.outputJson);
  fs.writeFileSync(args.outputJson, JSON.stringify(report, null, 2), "utf8");

  ensureDir(args.outputMd);
  fs.writeFileSync(args.outputMd, buildMarkdown(report), "utf8");

  console.log(`Wrote markdown report: ${args.outputMd}`);
  console.log(`Wrote JSON report: ${args.outputJson}`);
  console.log(
    `Summary: pass=${summary.pass}, observed=${summary.observed} (structural=${summary.observedStructural}, encoding=${summary.observedEncoding}), fail=${summary.fail}, unsupported=${summary.unsupported}, blocked=${summary.blocked}`,
  );

  if (args.strict && (summary.fail > 0 || summary.blocked > 0)) {
    process.exit(1);
  }
}

main().catch((err) => {
  console.error(err.message || err);
  process.exit(1);
});
