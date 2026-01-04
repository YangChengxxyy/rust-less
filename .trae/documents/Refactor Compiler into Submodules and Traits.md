我将把庞大的 `Compiler` 结构体拆分为多个功能模块，每个模块负责特定的编译逻辑。

### 1. 目录结构重构
- 创建 `src/compiler/` 目录
- 将 `src/compiler.rs` 移动并重命名为 `src/compiler/mod.rs`
- 在 `src/compiler/mod.rs` 中声明子模块

### 2. 定义功能 Trait (子模块)
我将创建以下子模块，并通过 Trait 为 `Compiler` 实现具体逻辑：

- **`src/compiler/expression.rs`**: `ExpressionCompiler` Trait
  - 负责 `evaluate_expression`, `evaluate_binary_op`, `resolve_variable` 等
  
- **`src/compiler/rule.rs`**: `RuleCompiler` Trait
  - 负责 `compile_rule`, `compile_selector`, `compile_declaration` 等
  
- **`src/compiler/mixin.rs`**: `MixinCompiler` Trait
  - 负责 `compile_mixin_definition`, `compile_mixin_call`, `register_rule_as_mixin` 等
  
- **`src/compiler/import.rs`**: `ImportCompiler` Trait
  - 负责 `compile_import`, `resolve_import_path` 等
  
- **`src/compiler/at_rule.rs`**: `AtRuleCompiler` Trait
  - 负责 `compile_at_rule`, `compile_media_query` 等

### 3. 修改 `Compiler` 定义
- 在 `src/compiler/mod.rs` 中，将 `Compiler` 的私有字段修改为 `pub(crate)`，以便子模块访问。
- 保留 `Compiler::new()`, `compile()` 等入口方法及 `push_scope`, `add_indent` 等基础辅助方法在 `mod.rs` 中。
- `compile_statement` 将作为调度中心，调用各 Trait 的方法。

### 4. 验证
- 运行所有测试用例，确保重构不破坏现有功能。
