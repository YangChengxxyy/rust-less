# 调试和测试工具目录

这个目录包含了开发 Rust LESS 编译器过程中使用的各种调试工具和测试文件。

## 📁 目录结构

```
debug/
├── programs/           # 调试程序
├── test_inputs/        # 测试用的 LESS 文件
├── README.md          # 本文档
├── debug_test.rs      # 通用调试程序
├── debug_string_test  # 字符串测试
└── debug_media.less   # 媒体查询调试文件
```

## 🔧 调试程序 (`programs/`)

### 词法分析调试
- **`debug_lexer.rs`** - 调试词法分析器，查看输入代码被分解成哪些 token
- **`debug_tokens.rs`** - 通用 token 分析工具
- **`debug_media_tokens.rs`** - 专门分析媒体查询的 token 化过程

### AST 调试
- **`debug_media_ast.rs`** - 调试媒体查询的 AST 结构

### 功能测试
- **`test_declaration.rs`** - 测试 CSS 声明的解析
- **`test_media_simple.rs`** - 测试简单媒体查询
- **`test_nested_media_simple.rs`** - 测试嵌套媒体查询
- **`test_final_media.rs`** - 完整的媒体查询功能演示

## 📄 测试输入文件 (`test_inputs/`)

### 基础功能测试
- **`test_simple.less`** - 基本 LESS 语法测试
- **`test_simple_rule.less`** / **`test_simple_rule2.less`** - 简单规则测试
- **`test_string.less`** - 字符串处理测试
- **`test_comment.less`** - 注释处理测试

### 媒体查询测试
- **`test_media.less`** / **`test_media2.less`** - 媒体查询基础测试
- **`test_nested_media.less`** - 嵌套媒体查询测试

### 复杂功能测试
- **`test_complex.less`** - 复杂 LESS 功能测试
- **`test_debug.less`** / **`test_debug2.less`** - 调试用测试文件

## 🚀 使用方法

### 编译调试程序
```bash
# 从项目根目录运行
rustc --edition 2021 -L target/debug/deps debug/programs/程序名.rs -o debug_程序 --extern rust_less=target/debug/librust_less.rlib
```

### 运行示例
```bash
# 调试词法分析
./debug_tokens

# 测试媒体查询 AST
./debug_media_ast

# 完整媒体查询功能演示
./test_final_media
```

## 📋 开发历程

这些文件记录了修复嵌套媒体查询功能的完整过程：

1. **问题发现** - 嵌套媒体查询编译输出不正确
2. **词法分析调试** - 确认 token 化正常
3. **AST 调试** - 发现解析器问题
4. **逐步修复** - 添加 `Statement::Declaration` 支持
5. **功能完善** - 改进格式化和编译逻辑
6. **最终验证** - 完整的功能测试

## 🧹 清理建议

在项目发布前，可以考虑：
- 保留 `test_final_media.rs` 作为功能演示
- 删除临时调试文件
- 将有价值的测试移到正式测试套件中

## 🎯 相关功能

主要解决的问题：
- ✅ 嵌套媒体查询正确编译
- ✅ 声明语句正确包装为规则
- ✅ 媒体查询提升到顶层
- ✅ 选择器上下文正确保持
- ✅ 格式化和空格处理

这个目录见证了一个复杂编译器功能从出现问题到最终修复的完整开发过程！ 🎉