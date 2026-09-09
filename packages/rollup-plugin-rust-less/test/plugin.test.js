import { test } from "node:test";
import assert from "node:assert/strict";
import { createRustLessPlugin } from "../index.js";

function makeStub(overrides = {}) {
  const calls = [];
  return {
    calls,
    async compileLessWithOptions(input, options) {
      calls.push({ input, options });
      if (overrides.reject) {
        throw new Error("wasm exploded");
      }
      if (overrides.error) {
        return { css: "", error: "NameError: undefined variable @x" };
      }
      return {
        css: `body{a:${input.length}}`,
        ...(overrides.sourceMap
          ? { sourceMap: '{"version":3,"mappings":"AAAA"}' }
          : {}),
      };
    },
  };
}

const failCtx = () => ({ error(msg) { throw new Error(msg); } });

test("transforms .less id into css", async () => {
  const stub = makeStub();
  const plugin = createRustLessPlugin(stub);
  const out = await plugin.transform.call(failCtx(), "a { color: red }", "src/main.less");
  assert.deepEqual(Object.keys(out), ["code", "map"]);
  assert.equal(out.code, "body{a:16}");
  assert.equal(plugin.name, "rollup-plugin-rust-less");
});

test("non-.less id returns null", async () => {
  const stub = makeStub();
  const plugin = createRustLessPlugin(stub);
  assert.equal(await plugin.transform.call(failCtx(), "body{}", "src/main.css"), null);
  assert.equal(stub.calls.length, 0);
});

test("forwards compress option to wasm", async () => {
  const stub = makeStub();
  await createRustLessPlugin(stub, { compress: true }).transform.call(failCtx(), "a{}", "a.less");
  assert.deepEqual(stub.calls[0].options, {
    compress: true,
    sourceMap: true,
    sourceMapLessjsCompat: false,
  });
});

test("error result calls this.error with file id and message", async () => {
  const stub = makeStub({ error: true });
  await assert.rejects(
    () => createRustLessPlugin(stub).transform.call(failCtx(), "@x;", "src/bad.less"),
    (err) => /src\/bad\.less/.test(err.message) && /undefined variable @x/.test(err.message)
  );
});

test("wasm rejection also routes through this.error", async () => {
  const stub = makeStub({ reject: true });
  await assert.rejects(
    () => createRustLessPlugin(stub).transform.call(failCtx(), "a{}", "src/bad.less"),
    /wasm exploded/
  );
});

test("sourceMap string is parsed into a map object", async () => {
  const stub = makeStub({ sourceMap: true });
  const out = await createRustLessPlugin(stub).transform.call(failCtx(), "a{}", "a.less");
  assert.deepEqual(out.map, { version: 3, mappings: "AAAA" });
});
