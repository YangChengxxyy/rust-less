# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2024-12-XX

### 🎉 新增功能 (Added)
- **颜色函数完整实现**: 
  - 实现了 `lighten()` 函数，支持HSL颜色空间亮度调整
  - 实现了 `darken()` 函数，支持HSL颜色空间亮度调整
  - 添加了完整的 RGB-HSL 颜色空间转换工具函数
  - 支持十六进制颜色和百分比参数的自动转换

- **API文档完善**:
  - 为所有公共 API 添加了详细的文档注释
  - 为 AST 模块、表达式和选择器添加了完整文档
  - 减少了 75% 的编译时文档警告

### 🔧 修复 (Fixed)
- **关键解析器错误修复**:
  - 修复了函数调用中变量参数的解析问题
  - 解决了 `lighten(@variable, 20%)` 等调用失败的问题
  - 修复了逗号列表解析与函数参数分隔符的冲突
  - 添加了智能前瞻逻辑，使解析更加上下文感知

- **表达式类型处理改进**:
  - 修复了 `Expression::Percentage` 类型在函数中的处理
  - 改进了十六进制颜色字符串到颜色表达式的转换
  - 优化了函数参数的类型检查和错误提示

### 📊 测试改进 (Tests)
- 集成测试通过率从 20/32 提升到 22/32 (+6.25%)
- 单元测试保持 95/95 (100% 通过)
- 新增颜色函数测试覆盖
- 修复了变量在函数参数中的测试用例

### 📚 文档更新 (Documentation)
- 更新了 README.md 以反映最新功能和状态
- 创建了详细的项目状态文档 (CURRENT_STATUS.md)
- 删除了过时和重复的文档文件
- 改进了功能演示示例，包含颜色函数用法

### 🏗️ 内部改进 (Internal)
- 代码可维护性显著提升
- 编译警告数量减少约 75%
- 改进了错误处理函数的参数验证
- 优化了 HSL 颜色计算的性能

## [0.2.0] - 2024-XX-XX

### 新增功能
- 基础的 LESS 语法支持
- 变量系统完整实现
- 选择器嵌套功能
- 父选择器引用 (&)
- 媒体查询嵌套
- 基础数学函数 (round, ceil, floor, percentage)
- 算术运算支持
- CSS 输出格式化

### 修复
- 初始解析器实现
- 基础错误处理
- 词法分析器优化

### 文档
- 项目 README 文档
- 基础 API 文档
- 安装和使用指南

## 计划中的版本

### [0.2.2] - 计划中 (1-2周内)
- 修复字符串函数中的变量参数解析问题
- 改进错误处理和错误信息
- 添加更多颜色函数 (saturate, desaturate, mix)
- 性能优化

### [0.3.0] - 计划中 (1-2个月内)
- 完整的混合器系统实现
- CSS 注释保留功能
- 基础的 :extend() 支持
- 改进的导入系统
- WebAssembly 支持

### [0.4.0] - 计划中 (3-6个月内)
- 变量插值 (@{variable})
- 完整的扩展功能
- 映射和列表支持
- 源码映射支持
- Language Server Protocol

---

## 版本说明

- **补丁版本** (0.x.Y): 错误修复和小的改进
- **次要版本** (0.X.y): 新功能添加，向后兼容
- **主要版本** (X.y.z): 重大变更，可能不向后兼容

## 链接

- [项目仓库](https://github.com/YangChengxxyy/rust-less)
- [问题追踪](https://github.com/YangChengxxyy/rust-less/issues)
- [功能请求](https://github.com/YangChengxxyy/rust-less/discussions)
- [文档](https://docs.rs/rust-less)