# Rust LESS 编译器架构图解

本文档包含了完整的 Rust LESS 编译器架构的 PlantUML 图表，从系统概览到详细的模块交互。

## 目录

1. [系统概览架构](#1-系统概览架构)
2. [编译流水线](#2-编译流水线)
3. [模块依赖关系](#3-模块依赖关系)
4. [AST 结构图](#4-ast-结构图)
5. [类关系图](#5-类关系图)
6. [编译器组件图](#6-编译器组件图)
7. [数据流图](#7-数据流图)
8. [错误处理架构](#8-错误处理架构)
9. [混合器系统架构](#9-混合器系统架构)
10. [变量插值系统](#10-变量插值系统)

---

## 1. 系统概览架构

```plantuml
@startuml System_Overview
!define RECTANGLE class

title Rust LESS 编译器 - 系统概览

package "外部接口" {
  [CLI 工具] as CLI
  [WASM 绑定] as WASM
  [Rust API] as API
}

package "核心编译器" {
  [编译器主控] as Compiler
  [词法分析器] as Lexer
  [语法解析器] as Parser
  [函数库] as Functions
}

package "AST 系统" {
  [表达式] as Expressions
  [选择器] as Selectors
  [语句] as Statements
  [值类型] as Values
}

package "工具模块" {
  [错误处理] as Error
  [作用域管理] as Scope
}

CLI --> API : 调用
WASM --> API : 调用
API --> Compiler : compile()

Compiler --> Lexer : tokenize()
Compiler --> Parser : parse()
Compiler --> Functions : 函数调用

Lexer --> Parser : Token 流
Parser --> Statements : 构建 AST
Parser --> Expressions : 构建表达式
Parser --> Selectors : 构建选择器

Compiler --> Scope : 管理变量作用域
Compiler --> Error : 错误报告

note right of Compiler
  主要编译流程:
  1. 词法分析 (Tokenization)
  2. 语法解析 (Parsing) 
  3. AST 构建
  4. 编译生成 CSS
end note

@enduml
```

---

## 2. 编译流水线

```plantuml
@startuml Compilation_Pipeline
!define RECTANGLE class

title LESS 编译流水线 - 详细流程

start

:LESS 源代码;
note right: 输入字符串

:词法分析 (Lexer);
note right
  将源代码转换为 Token 流
  - 识别关键字、操作符
  - 处理字符串、数字
  - 识别变量、混合器语法
end note

:Token 流;

:语法分析 (Parser);
note right
  构建抽象语法树 (AST)
  - 解析变量声明
  - 解析混合器定义
  - 解析规则和选择器
  - 处理嵌套结构
end note

:AST (语法树);

:语义分析;
note right
  验证语法正确性
  - 检查未定义变量
  - 验证混合器调用
  - 类型检查
end note

if (存在错误?) then (是)
  :生成错误报告;
  stop
else (否)
  :代码生成;
  note right
    遍历 AST 生成 CSS
    - 展开混合器
    - 计算表达式
    - 处理嵌套规则
    - 应用变量插值
  end note
endif

:CSS 输出;
note right: 最终的 CSS 字符串

stop

@enduml
```

---

## 3. 模块依赖关系

```plantuml
@startuml Module_Dependencies
!define RECTANGLE class

title 模块依赖关系图

package "lib.rs" {
  component [主入口] as Main
}

package "词法分析" {
  component [lexer.rs] as Lexer
}

package "语法分析" {
  component [parser.rs] as Parser
}

package "编译器" {
  component [compiler.rs] as Compiler
}

package "AST 模块" {
  component [ast/mod.rs] as ASTMod
  component [ast/expressions.rs] as Expressions
  component [ast/selectors.rs] as Selectors
  component [ast/values.rs] as Values
}

package "功能模块" {
  component [functions.rs] as Functions
  component [error.rs] as Error
}

package "二进制工具" {
  component [bin/*.rs] as BinTools
}

package "WASM 支持" {
  component [wasm.rs] as WASM
}

' 依赖关系
Main --> Compiler
Main --> Error
Main --> WASM

Compiler --> Lexer
Compiler --> Parser
Compiler --> Functions
Compiler --> Error
Compiler --> ASTMod

Parser --> Lexer
Parser --> ASTMod
Parser --> Error

Lexer --> Error

ASTMod --> Expressions
ASTMod --> Selectors
ASTMod --> Values

BinTools --> Main
WASM --> Main

Functions --> Expressions
Functions --> Error

note as N1
  依赖层次:
  1. 底层: Error, AST 组件
  2. 中层: Lexer, Parser, Functions
  3. 上层: Compiler
  4. 接口层: Main, WASM, BinTools
end note

@enduml
```

---

## 4. AST 结构图

```plantuml
@startuml AST_Structure
!define RECTANGLE class

title AST (抽象语法树) 结构

abstract class Statement {
  +position: Position
}

class VariableDeclaration {
  +name: String
  +value: Expression
  +default: bool
}

class Rule {
  +selectors: Vec<Selector>
  +declarations: Vec<Declaration>
  +nested_rules: Vec<Statement>
}

class Declaration {
  +property: String
  +value: Expression
  +important: bool
}

class MixinDefinition {
  +name: String
  +parameters: Vec<MixinParameter>
  +guard: Option<Expression>
  +body: Vec<Statement>
}

class MixinCall {
  +name: String
  +arguments: Vec<Expression>
  +important: bool
}

class Import {
  +path: String
  +import_type: ImportType
  +media: Option<String>
}

class AtRule {
  +name: String
  +prelude: Option<String>
  +block: Option<Vec<Statement>>
}

Statement <|-- VariableDeclaration
Statement <|-- Rule
Statement <|-- Declaration
Statement <|-- MixinDefinition
Statement <|-- MixinCall
Statement <|-- Import
Statement <|-- AtRule

abstract class Expression {
  +position: Position
}

class StringExpr {
  +value: String
  +quoted: bool
}

class NumberExpr {
  +value: f64
  +unit: Option<String>
}

class ColorExpr {
  +red: u8
  +green: u8
  +blue: u8
  +alpha: f64
}

class VariableExpr {
  +name: String
}

class BinaryOpExpr {
  +left: Box<Expression>
  +operator: BinaryOperator
  +right: Box<Expression>
}

class FunctionCallExpr {
  +name: String
  +arguments: Vec<Expression>
}

class InterpolationExpr {
  +variable: String
}

Expression <|-- StringExpr
Expression <|-- NumberExpr
Expression <|-- ColorExpr
Expression <|-- VariableExpr
Expression <|-- BinaryOpExpr
Expression <|-- FunctionCallExpr
Expression <|-- InterpolationExpr

Declaration --> Expression : value
VariableDeclaration --> Expression : value
BinaryOpExpr --> Expression : left, right
FunctionCallExpr --> Expression : arguments

@enduml
```

---

## 5. 类关系图

```plantuml
@startuml Class_Relationships
!define RECTANGLE class

title 核心类关系图

class Lexer {
  -input: String
  -position: usize
  -line: usize
  -column: usize
  +new(input: String): Lexer
  +tokenize(): Result<Vec<Token>>
  +next_token(): Result<Token>
}

class Parser {
  -tokens: Vec<Token>
  -current: usize
  +new(lexer: Lexer): Result<Parser>
  +parse(): Result<Stylesheet>
  +parse_statement(): Result<Statement>
  +parse_expression(): Result<Expression>
}

class Compiler {
  -scope_stack: Vec<Scope>
  -output: String
  -indent_level: usize
  +new(): Compiler
  +compile(input: &str): Result<String>
  +compile_statement(stmt: &Statement): Result<()>
  +evaluate_expression(expr: &Expression): Result<Expression>
}

class Scope {
  +variables: HashMap<String, Expression>
  +mixins: HashMap<String, Vec<MixinDefinition>>
  +parent: Option<Box<Scope>>
  +define_variable(name: String, value: Expression)
  +lookup_variable(name: &str): Option<&Expression>
  +define_mixin(name: String, mixin: MixinDefinition)
  +lookup_mixin(name: &str): Option<&Vec<MixinDefinition>>
}

class Token {
  +token_type: TokenType
  +position: Position
  +lexeme: String
}

class Position {
  +line: usize
  +column: usize
}

enum TokenType {
  String(String)
  Number(f64)
  Identifier(String)
  AtKeyword(String)
  When
  Ellipsis
  LeftParen
  RightParen
  ...
}

class Error {
  +message: String
  +line: usize
  +column: usize
  +error_type: ErrorType
}

Lexer --> Token : 生成
Parser --> Lexer : 使用
Parser --> Error : 可能抛出
Compiler --> Parser : 使用
Compiler --> Scope : 管理
Compiler --> Error : 可能抛出

Token --> TokenType : 包含
Token --> Position : 包含
Error --> Position : 包含

Scope --> Expression : 存储变量
Scope --> MixinDefinition : 存储混合器

@enduml
```

---

## 6. 编译器组件图

```plantuml
@startuml Compiler_Components
!define RECTANGLE class

title 编译器内部组件图

package "编译器核心" as CompilerCore {
  
  component "主编译器" as MainCompiler {
    class Compiler {
      +compile(input: &str): Result<String>
      +compile_statement(stmt: &Statement): Result<()>
      +compile_rule(rule: &Rule, parent_selectors: &[String]): Result<()>
      +compile_declaration(decl: &Declaration): Result<()>
    }
  }
  
  component "表达式求值器" as ExprEvaluator {
    class ExpressionEvaluator {
      +evaluate_expression(expr: &Expression): Result<Expression>
      +evaluate_binary_op(left: &Expr, op: &Op, right: &Expr): Result<Expression>
      +evaluate_function_call(name: &str, args: &[Expr]): Result<Expression>
    }
  }
  
  component "混合器处理器" as MixinProcessor {
    class MixinProcessor {
      +compile_mixin_definition(mixin: &MixinDefinition): Result<()>
      +compile_mixin_call(call: &MixinCall): Result<()>
      +find_matching_mixin(defs: &[MixinDef], call: &MixinCall): Result<Option<MixinDef>>
      +bind_mixin_arguments(mixin: &MixinDef, call: &MixinCall, scope: &mut Scope): Result<()>
    }
  }
  
  component "作用域管理器" as ScopeManager {
    class ScopeManager {
      +push_scope(): ()
      +pop_scope(): ()
      +current_scope(): &mut Scope
      +define_variable(name: String, value: Expression): ()
      +lookup_variable(name: &str): Option<&Expression>
    }
  }
  
  component "选择器处理器" as SelectorProcessor {
    class SelectorProcessor {
      +compile_selector(selector: &Selector, parent: &[String]): Vec<String>
      +handle_parent_selector(selector: &str, parent: &[String]): String
      +resolve_interpolation(selector: &str, scope: &Scope): Result<String>
    }
  }
  
  component "CSS 生成器" as CSSGenerator {
    class CSSGenerator {
      +add_indent(): ()
      +add_space(): ()
      +add_newline(): ()
      +output_selector(selectors: &[String]): ()
      +output_declaration(property: &str, value: &str): ()
    }
  }
}

package "支持组件" as SupportComponents {
  
  component "函数库" as FunctionLibrary {
    class Functions {
      +round(value: f64): f64
      +percentage(value: f64): String
      +rgb(r: u8, g: u8, b: u8): Color
      +lighten(color: Color, amount: f64): Color
    }
  }
  
  component "错误处理器" as ErrorHandler {
    class ErrorReporter {
      +semantic_error(msg: &str, line: usize, column: usize): Error
      +type_mismatch(expected: &str, found: &str, line: usize, column: usize): Error
      +undefined_variable(name: &str, line: usize, column: usize): Error
    }
  }
}

MainCompiler --> ExprEvaluator : 使用
MainCompiler --> MixinProcessor : 使用
MainCompiler --> ScopeManager : 使用
MainCompiler --> SelectorProcessor : 使用
MainCompiler --> CSSGenerator : 使用

ExprEvaluator --> FunctionLibrary : 调用函数
MixinProcessor --> ScopeManager : 管理作用域
SelectorProcessor --> ScopeManager : 查找变量

MainCompiler --> ErrorHandler : 报告错误
ExprEvaluator --> ErrorHandler : 报告错误
MixinProcessor --> ErrorHandler : 报告错误

@enduml
```

---

## 7. 数据流图

```plantuml
@startuml Data_Flow
!define RECTANGLE class

title 数据流转图

start

:LESS 源码;
note right: 字符串输入

partition "词法分析阶段" {
  :逐字符扫描;
  :识别 Token 类型;
  :生成 Token 流;
}

:Token 序列;
note right
  TokenType 枚举:
  - String, Number, Identifier
  - AtKeyword (@variable)
  - When, Ellipsis (...)
  - LeftParen, RightParen
  - 等等...
end note

partition "语法分析阶段" {
  :解析语句;
  split
    :变量声明;
  split again
    :混合器定义;
  split again
    :CSS 规则;
  split again
    :混合器调用;
  end split
  :构建 AST 节点;
}

:AST 树结构;
note right
  Statement 层次:
  - VariableDeclaration
  - MixinDefinition
  - MixinCall
  - Rule
  - Declaration
end note

partition "编译阶段" {
  :遍历 AST;
  
  fork
    :处理变量;
    :存储到作用域;
  fork again
    :处理混合器定义;
    :注册到作用域;
  fork again
    :处理混合器调用;
    :查找并展开;
  fork again
    :处理 CSS 规则;
    :生成选择器和声明;
  end fork
  
  :合并处理结果;
}

:CSS 输出;
note right: 最终的 CSS 字符串

stop

note as DataTypes
  **关键数据类型转换:**
  
  String → Vec<Token>
  Vec<Token> → Stylesheet (AST)
  Stylesheet → String (CSS)
  
  **中间数据结构:**
  - Expression (表达式树)
  - Selector (选择器树)
  - Scope (作用域栈)
end note

@enduml
```

---

## 8. 错误处理架构

```plantuml
@startuml Error_Handling
!define RECTANGLE class

title 错误处理架构图

abstract class Error {
  +message: String
  +line: usize
  +column: usize
}

class LexError {
  +message: String
  +position: Position
}

class ParseError {
  +message: String
  +expected: String
  +found: String
  +position: Position
}

class SemanticError {
  +message: String
  +context: String
  +position: Position
}

class UndefinedVariableError {
  +name: String
  +position: Position
}

class UndefinedMixinError {
  +name: String
  +position: Position
}

class TypeMismatchError {
  +expected: String
  +found: String
  +position: Position
}

class DivisionByZeroError {
  +position: Position
}

Error <|-- LexError
Error <|-- ParseError
Error <|-- SemanticError
Error <|-- UndefinedVariableError
Error <|-- UndefinedMixinError
Error <|-- TypeMismatchError
Error <|-- DivisionByZeroError

component "词法分析器" as Lexer {
  class LexerImpl {
    +tokenize(): Result<Vec<Token>, LexError>
  }
}

component "语法分析器" as Parser {
  class ParserImpl {
    +parse(): Result<Stylesheet, ParseError>
  }
}

component "编译器" as Compiler {
  class CompilerImpl {
    +compile(): Result<String, SemanticError>
  }
}

LexerImpl --> LexError : 可能产生
ParserImpl --> ParseError : 可能产生
CompilerImpl --> SemanticError : 可能产生
CompilerImpl --> UndefinedVariableError : 可能产生
CompilerImpl --> UndefinedMixinError : 可能产生
CompilerImpl --> TypeMismatchError : 可能产生
CompilerImpl --> DivisionByZeroError : 可能产生

note as ErrorFlow
  **错误传播流程:**
  
  1. 底层组件产生具体错误
  2. 错误向上传播到调用者
  3. 顶层统一处理和报告
  
  **错误恢复策略:**
  - 词法错误: 停止分析
  - 语法错误: 尝试同步到下一个语句
  - 语义错误: 收集所有错误后报告
end note

@enduml
```

---

## 9. 混合器系统架构

```plantuml
@startuml Mixin_System
!define RECTANGLE class

title 混合器系统架构

package "混合器定义" {
  class MixinDefinition {
    +name: String
    +parameters: Vec<MixinParameter>
    +guard: Option<Expression>
    +body: Vec<Statement>
  }
  
  class MixinParameter {
    +name: String
    +default_value: Option<Expression>
    +variadic: bool
  }
}

package "混合器调用" {
  class MixinCall {
    +name: String
    +arguments: Vec<Expression>
    +important: bool
  }
}

package "混合器处理" {
  class MixinRegistry {
    +mixins: HashMap<String, Vec<MixinDefinition>>
    +register_mixin(mixin: MixinDefinition)
    +lookup_mixins(name: &str): Option<&Vec<MixinDefinition>>
  }
  
  class MixinMatcher {
    +find_matching_mixin(defs: &[MixinDefinition], call: &MixinCall): Option<MixinDefinition>
    +evaluate_guard(guard: &Expression, scope: &Scope): Result<bool>
    +check_parameter_compatibility(params: &[MixinParam], args: &[Expression]): bool
  }
  
  class MixinExpander {
    +expand_mixin(mixin: &MixinDefinition, call: &MixinCall): Result<Vec<Statement>>
    +bind_arguments(params: &[MixinParam], args: &[Expression]): Result<Scope>
    +expand_variadic_args(param: &MixinParam, args: &[Expression]): Expression
  }
}

package "守卫系统" {
  class GuardEvaluator {
    +evaluate_condition(expr: &Expression, scope: &Scope): Result<bool>
    +evaluate_comparison(left: &Expression, op: ComparisonOp, right: &Expression): Result<bool>
  }
  
  enum ComparisonOperator {
    Equal
    NotEqual
    LessThan
    LessThanOrEqual
    GreaterThan
    GreaterThanOrEqual
  }
}

MixinDefinition --> MixinParameter : 包含
MixinDefinition --> Expression : guard

MixinRegistry --> MixinDefinition : 存储
MixinMatcher --> MixinRegistry : 查询
MixinMatcher --> GuardEvaluator : 使用
MixinExpander --> MixinMatcher : 使用

GuardEvaluator --> ComparisonOperator : 使用

note as MixinFlow
  **混合器处理流程:**
  
  1. **定义阶段**: 解析并注册混合器到 Registry
  2. **调用阶段**: 查找匹配的混合器定义
  3. **匹配阶段**: 评估守卫条件，选择正确的重载
  4. **展开阶段**: 绑定参数，展开混合器体
  
  **守卫系统特性:**
  - 支持条件表达式 when (@var > 10)
  - 支持多个同名混合器不同条件
  - 运行时动态选择匹配的混合器
end note

@enduml
```

---

## 10. 变量插值系统

```plantuml
@startuml Variable_Interpolation
!define RECTANGLE class

title 变量插值系统架构

package "插值语法" {
  enum InterpolationType {
    SelectorInterpolation
    PropertyInterpolation  
    ValueInterpolation
  }
  
  class InterpolationExpression {
    +variable: String
    +interpolation_type: InterpolationType
    +position: Position
  }
}

package "插值处理器" {
  class InterpolationProcessor {
    +process_selector_interpolation(selector: &str, scope: &Scope): Result<String>
    +process_property_interpolation(property: &str, scope: &Scope): Result<String>
    +process_value_interpolation(value: &Expression, scope: &Scope): Result<Expression>
  }
  
  class InterpolationParser {
    +parse_interpolation_syntax(input: &str): Vec<InterpolationPart>
    +extract_variable_names(parts: &[InterpolationPart]): Vec<String>
  }
  
  class InterpolationEvaluator {
    +evaluate_interpolation(var_name: &str, scope: &Scope): Result<String>
    +format_interpolated_value(value: &Expression): String
  }
}

package "插值数据结构" {
  enum InterpolationPart {
    Literal(String)
    Variable(String)
  }
  
  class TemplateString {
    +parts: Vec<InterpolationPart>
    +position: Position
  }
}

package "应用场景" {
  class SelectorInterpolation {
    +template: String
    +variables: Vec<String>
    ' 例如: .@{prefix}-button
  }
  
  class PropertyInterpolation {
    +template: String  
    +variables: Vec<String>
    ' 例如: @{property}: value
  }
  
  class ValueInterpolation {
    +template: Expression
    +variables: Vec<String>
    ' 例如: url("@{base-path}/image.png")
  }
}

InterpolationExpression --> InterpolationType
TemplateString --> InterpolationPart

InterpolationProcessor --> InterpolationParser : 使用
InterpolationProcessor --> InterpolationEvaluator : 使用

InterpolationParser --> InterpolationPart : 生成
InterpolationParser --> TemplateString : 生成

SelectorInterpolation --> InterpolationProcessor : 使用
PropertyInterpolation --> InterpolationProcessor : 使用  
ValueInterpolation --> InterpolationProcessor : 使用

note as InterpolationFlow
  **插值处理流程:**
  
  1. **词法阶段**: 识别 @{variable} 语法
  2. **解析阶段**: 构建插值表达式树
  3. **编译阶段**: 
     - 查找变量值
     - 替换插值部分
     - 生成最终字符串
  
  **支持的插值类型:**
  - 选择器: .@{selector} { }
  - 属性名: @{property}: value;
  - 属性值: color: @{color};
  - URL: url("@{path}/file.png")
  
  **特殊处理:**
  - 嵌套插值支持
  - 类型安全的值转换
  - 错误处理和回退机制
end note

@enduml
```

---

## 使用说明

### 如何查看这些图表

1. **在线查看**: 将 PlantUML 代码复制到 [PlantUML Online Server](http://www.plantuml.com/plantuml/uml/)
2. **本地生成**: 
   ```bash
   # 安装 PlantUML
   npm install -g node-plantuml
   
   # 生成图片
   puml generate architecture_diagrams.md --format png
   ```
3. **IDE 插件**: 在 VS Code、IntelliJ 等 IDE 中安装 PlantUML 插件

### 图表说明

- **系统概览**: 展示了整个编译器的高层架构和模块关系
- **编译流水线**: 详细描述了从 LESS 到 CSS 的编译过程
- **模块依赖**: 显示了各 Rust 模块之间的依赖关系
- **AST 结构**: 抽象语法树的详细类图
- **类关系**: 核心类之间的关系和交互
- **编译器组件**: 编译器内部组件的详细设计
- **数据流**: 数据在编译过程中的流转
- **错误处理**: 错误处理机制的架构设计
- **混合器系统**: LESS 混合器功能的完整架构
- **变量插值**: 变量插值系统的设计和实现

这些图表提供了从宏观到微观的完整视角，帮助理解 Rust LESS 编译器的架构设计和实现细节。