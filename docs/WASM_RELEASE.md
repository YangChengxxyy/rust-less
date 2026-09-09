# WASM 发布流程（标准化）

本文档描述 rust-less WebAssembly 产物的一键构建、冒烟测试与 npm 发布流程。
对应脚本：仓库根目录的 `./build-wasm.sh`；CI 定义：`.github/workflows/wasm-release.yml`。

## 目标与产物

| 目录 | npm 包名 | 受众 |
| --- | --- | --- |
| `pkg/` | `rust-less-wasm` | 浏览器 / bundler（webpack、vite 等），**主发布目标**，含完整异步包装 API |
| `pkg-nodejs/` | `rust-less-wasm-node` | Node.js（>= 18），供构建工具插件（webpack/vite/rollup 的 rust-less 插件）`await import()` 懒加载使用 |
| `pkg-web/` | `rust-less-wasm-web` | 无法使用打包器、直接 `<script>` / ESM CDN 引入的 Web 场景（次要） |

三个包暴露同一个异步 API（详见各包 `index.d.ts`）：

- `compileLess(input) -> Promise<{css, sourceMap?, error?}>`
- `compileLessWithOptions(input, {compress?, sourceMap?, sourceMapLessjsCompat?})`
- `createCompiler(options?) -> Promise<{compile(input)}>`
- `validateLess(input)` / `getVersion()` / `getInfo()` / `getErrorDetails(input)`

编译失败不抛出异常，返回 `{ css: "", error: "<信息>" }`；启用 `sourceMap` 选项时结果包含
`sourceMap`（JSON 字符串，`version: 3`）。

## 前置条件

| 工具 | 版本约束 | 安装方式 |
| --- | --- | --- |
| Rust 工具链 | stable，含 `wasm32-unknown-unknown` 目标 | `rustup target add wasm32-unknown-unknown` |
| wasm-pack | 0.13.1 | `cargo install wasm-pack --version 0.13.1` |
| wasm-bindgen-cli | **0.2.100（必须与 Cargo.lock 中 wasm-bindgen 一致）** | `cargo install wasm-bindgen-cli --version 0.2.100` |
| node | >= 18 | https://nodejs.org/ |
| jq | 任意近期版本 | `brew install jq` 等 |

网络受限环境的回退：GitHub release 下载被阻止时，所有工具都可以用
`cargo install <crate> --version <版本>` 从 crates.io 源码安装；脚本固定使用
`wasm-pack build --mode no-install`，避免 wasm-pack 在构建时联网下载 wasm-bindgen /
wasm-opt，改用 PATH 上已安装的同版本二进制。

## 一键流程

```bash
./build-wasm.sh                    # 完整构建三个目标 + 后处理 + 冒烟测试
./build-wasm.sh --skip-build       # 复用已有 pkg*/ 产物，仅重新后处理 + 冒烟
./build-wasm.sh --smoke            # 仅对 pkg-nodejs 运行冒烟测试
./build-wasm.sh -h                 # 帮助
```

脚本内部步骤：

1. 工具与 wasm32 目标检查（缺失时报错并给出安装方式，目标缺失时自动 `rustup target add`）；
2. 通过 `cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].version'` 读取 crate 版本；
3. 清理旧 `pkg*/`，依次以 `--target bundler / nodejs / web` 构建（`--mode no-install --features wasm`，
   `--features wasm` 必须放在 `--` 之后，否则会被误传给 `cargo build`）；
4. 对每个目录写入包装 API（`index.js` / `index.cjs` / `index.mjs`）、`index.d.ts`、
   `package.json`、中文 `README.md`；
5. 每个目录写入 `VERSION` 与 `SHA256SUMS.txt`（有仓库根 LICENSE 时一并拷贝）；
6. 运行内置冒烟测试（见下），全部通过才退出 0。

### 冒烟测试内容（针对 pkg-nodejs）

- `compileLess('@c:#333;.a{color:@c}')` → css 含 `.a` 与 `#333`；
- `{compress:true}` → 输出含 `.a{`（压缩形态）；
- `{sourceMap:true}` → `sourceMap` 可解析为 JSON 且 `version === 3`；
- 错误输入 `.a{color:@nope}` → `error` 非空且提及变量名，`css === ""`；
- `getVersion()` 与 crate 版本一致；
- `import('./pkg-nodejs/index.mjs')` ESM 入口可用。

## 版本规则

- **npm 包版本 == crate 版本（Cargo.toml `version`）**，锁步发布；
- 脚本动态读取版本，构建产物中 `package.json` 的 `version` 与 `VERSION` 文件均来自该值；
- 发新版本时：先升 Cargo.toml → 打 `v*` tag → CI 构建 → 三包同版本发布。

## 发布步骤

```bash
# 1. 构建 + 冒烟（本地必须全绿）
./build-wasm.sh

# 2. 预览每个包的内容（核对文件列表与体积）
(cd pkg && npm pack --dry-run)
(cd pkg-nodejs && npm pack --dry-run)
(cd pkg-web && npm pack --dry-run)

# 3. 按顺序发布：先主包，再 Node 变体与 web 变体
(cd pkg && npm publish)
(cd pkg-nodejs && npm publish)
(cd pkg-web && npm publish)
```

首次发布需 `npm login`；发布后可用 `SHA256SUMS.txt` 供使用者校验 wasm 文件。

## CI 集成

`.github/workflows/wasm-release.yml`：

- **触发**：push `v*` tag；
- **build 作业**：ubuntu-latest，安装 Rust + wasm32 目标，用
  `taiki-e/install-action@v2` 安装 wasm-pack 与 wasm-bindgen-cli，运行 `./build-wasm.sh`，
  将 `pkg/ pkg-nodejs/ pkg-web/` 上传为 `pkg-bundle` artifact；
- **publish 作业**：依赖 build，仅在 tag 上运行；通过 `actions/setup-node@v4` 配置
  npm registry，仅当配置了 `secrets.NPM_TOKEN` 时执行 `npm publish`（未配置 token 时
  跳过发布，便于 fork / 测试运行）。

## 故障排查

| 症状 | 原因与处理 |
| --- | --- |
| 构建报 `wasm-bindgen` 版本不匹配（`installed X.Y.Z but Y.Z.W expected`） | PATH 上的 wasm-bindgen-cli 与 Cargo.lock 不一致。执行 `cargo install wasm-bindgen-cli --version 0.2.100 --force` 后重试；脚本使用 `--mode no-install`，因此不会自动下载，必须手动对齐 |
| `error: unexpected argument '--out-dir' found`（来自 cargo build） | `--features wasm` 被放在了 wasm-pack 自身参数之前，导致后续参数被当作 cargo 参数转发。保持脚本中的顺序：`wasm-pack build --mode no-install --target <t> --out-dir <dir> -- --features wasm` |
| 日志出现 `wasm-opt` not found / Skipping wasm-opt | 可忽略——`--mode no-install` 跳过优化器下载；产物未做 wasm-opt 后处理，功能不受影响 |
| 冒烟测试 `getVersion()` 不一致 | `pkg*/` 产物是旧版本构建的，去掉 `--skip-build` 重新完整构建 |
| `cargo metadata` 报 Cargo.toml 解析错误 | Cargo.toml 正在被并行编辑或语法损坏，修复后重试 |
