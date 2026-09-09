import { test } from "node:test";
import assert from "node:assert/strict";
import { createLoader } from "../index.js";

// 符合 rust-less-wasm-node 契约的桩实现（无需安装真实包）。
function makeStub(overrides = {}) {
  const calls = [];
  const stub = {
    calls,
    async compileLessWithOptions(input, options) {
      calls.push({ input, options });
      if (overrides.reject) {
        throw new Error("wasm exploded");
      }
      if (overrides.error) {
        return { css: "", error: "ParseError: unclosed block on line 3" };
      }
      return {
        css: `body{a:${input.length}}`,
        ...(overrides.sourceMap
          ? { sourceMap: '{"version":3,"mappings":"AAAA"}' }
          : {}),
      };
    },
  };
  return stub;
}

function makeCtx(opts = {}, extra = {}) {
  const ctx = {
    resourcePath: "src/style.less",
    sourceMap: false,
    getOptions: () => opts,
    callback: (err, code, map) => {
      ctx.result = { err, code, map };
    },
    ...extra,
  };
  return ctx;
}

function run(stub, ctx, source = "a { b: c }") {
  const loader = createLoader(stub);
  return loader.call(ctx, source);
}

test("passes compiled css through to callback", async () => {
  const stub = makeStub();
  const ctx = makeCtx();
  await run(stub, ctx);
  assert.equal(ctx.result.err, null);
  assert.equal(ctx.result.code, "body{a:10}");
  assert.equal(ctx.result.map, null);
});

test("forwards compress option to wasm", async () => {
  const stub = makeStub();
  const ctx = makeCtx({ compress: true });
  await run(stub, ctx);
  assert.deepEqual(stub.calls[0].options, {
    compress: true,
    sourceMap: false,
    sourceMapLessjsCompat: false,
  });
});

test("falls back to this.sourceMap when getOptions absent", async () => {
  const stub = makeStub();
  const ctx = makeCtx();
  delete ctx.getOptions;
  ctx.sourceMap = true;
  await run(stub, ctx);
  assert.equal(stub.calls[0].options.sourceMap, true);
});

test("error result routes to callback with file id and message", async () => {
  const stub = makeStub({ error: true });
  const ctx = makeCtx();
  await run(stub, ctx);
  assert.ok(ctx.result.err instanceof Error);
  assert.match(ctx.result.err.message, /src\/style\.less/);
  assert.match(ctx.result.err.message, /unclosed block on line 3/);
});

test("rejections from the wasm runtime are routed to callback", async () => {
  const stub = makeStub({ reject: true });
  const ctx = makeCtx();
  await run(stub, ctx);
  assert.ok(ctx.result.err instanceof Error);
  assert.match(ctx.result.err.message, /wasm exploded/);
});

test("sourceMap string is parsed into a map object", async () => {
  const stub = makeStub({ sourceMap: true });
  const ctx = makeCtx({ sourceMap: true });
  await run(stub, ctx);
  assert.deepEqual(ctx.result.map, { version: 3, mappings: "AAAA" });
});
