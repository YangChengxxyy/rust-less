# Rust LESS 编译器时序图集

本文档包含 Rust LESS 编译器的详细时序图，展示各组件之间的交互流程。

## 目录

1. [完整编译流程时序图](#1-完整编译流程时序图)
2. [混合器处理时序图](#2-混合器处理时序图)
3. [变量解析时序图](#3-变量解析时序图)
4. [错误处理时序图](#4-错误处理时序图)
5. [作用域管理时序图](#5-作用域管理时序图)
6. [守卫条件评估时序图](#6-守卫条件评估时序图)

---

## 1. 完整编译流程时序图

```plantuml
@startuml Complete_Compilation_Flow
!define RECTANGLE class

title 完整 LESS 编译流程时序图

actor "用户" as User
participant "编译器 API" as API
participant "词法分析器" as Lexer
participant "语法分析器" as Parser
participant "编译器核心" as Compiler
participant "作用域管理" as Scope
participant "表达式求值" as Evaluator
participant "CSS 生成器" as Generator

User -> API: compile(less_source)
activate API

API -> Lexer: new(less_source)
activate Lexer
Lexer -> Lexer: 初始化词法分析器
Lexer --> API: lexer 实例
deactivate Lexer

API -> Lexer: tokenize()
activate Lexer
loop 逐字符扫描
    Lexer -> Lexer: next_token()
    note right: 识别 Token 类型\n处理字符串、数字、关键字
end
Lexer --> API: Vec<Token>
deactivate Lexer

API -> Parser: new(tokens)
activate Parser
Parser -> Parser: 初始化解析器状态
Parser --> API: parser 实例
deactivate Parser

API -> Parser: parse()
activate Parser
loop 遍历 Token 流
    Parser -> Parser: parse_statement()
    alt 变量声明
        Parser -> Parser: parse_variable_declaration()
    else 混合器定义
        Parser -> Parser: parse_mixin_definition()
    else CSS 规则
        Parser -> Parser: parse_rule()
    else 混合器调用
        Parser -> Parser: parse_mixin_call()
    end
end
Parser --> API: Stylesheet (AST)
deactivate Parser

API -> Compiler: new()
activate Compiler
Compiler -> Scope: new()
activate Scope
Scope --> Compiler: 初始作用域
deactivate Scope
Compiler --> API: compiler 实例
deactivate Compiler

API -> Compiler: compile_ast(stylesheet)
activate Compiler

loop 遍历 AST 语句
    Compiler -> Compiler: compile_statement(stmt)
    
    alt 变量声明
        Compiler -> Scope: define_variable(name, value)
        activate Scope
        Scope --> Compiler: 变量已存储
        deactivate Scope
        
    else 混合器定义
        Compiler -> Scope: define_mixin(name, definition)
        activate Scope
        Scope --> Compiler: 混合器已注册
        deactivate Scope
        
    else 混合器调用
        Compiler -> Scope: lookup_mixin(name)
        activate Scope
        Scope --> Compiler: Vec<MixinDefinition>
        deactivate Scope
        
        Compiler -> Compiler: find_matching_mixin(definitions, call)
        Compiler -> Compiler: expand_mixin(matched_mixin, call)
        
    else CSS 规则
        Compiler -> Generator: output_rule(selectors, declarations)
        activate Generator
        
        loop 处理每个声明
            Compiler -> Evaluator: evaluate_expression(value)
            activate Evaluator
            Evaluator --> Compiler: 求值结果
            deactivate Evaluator
            
            Generator -> Generator: add_declaration(property, value)
        end
        
        Generator --> Compiler: 规则已输出
        deactivate Generator
    end
end

Compiler --> API: CSS 字符串
deactivate Compiler

API --> User: 编译结果
deactivate API

@enduml
```

---

## 2. 混合器处理时序图

```plantuml
@startuml Mixin_Processing_Flow
!define RECTANGLE class

title 混合器定义和调用处理流程

participant "解析器" as Parser
participant "编译器" as Compiler
participant "作用域" as Scope
participant "混合器匹配器" as Matcher
participant "守卫求值器" as GuardEval
participant "参数绑定器" as ParamBinder

note over Parser, ParamBinder: 混合器定义阶段

Parser -> Parser: 检测到混合器定义语法
Parser -> Parser: parse_mixin_definition()

alt 带参数的混合器
    Parser -> Parser: parse_mixin_parameters()
    loop 解析每个参数
        Parser -> Parser: parse_parameter(@param)
        alt 可变参数
            Parser -> Parser: 检测 "..." 语法
            note right: @args... 语法
        else 默认参数
            Parser -> Parser: 解析默认值表达式
            note right: @param: default_value
        end
    end
end

alt 带守卫的混合器
    Parser -> Parser: 检测 "when" 关键字
    Parser -> Parser: parse_guard_expression()
    note right: when (@param > 10)
end

Parser -> Parser: parse_mixin_body()
Parser -> Compiler: MixinDefinition 完成

Compiler -> Scope: define_mixin(name, definition)
Scope -> Scope: 存储到 HashMap<String, Vec<MixinDefinition>>
note right: 支持同名混合器的多个重载

note over Parser, ParamBinder: 混合器调用阶段

Parser -> Parser: 检测到混合器调用语法
Parser -> Parser: parse_mixin_call()
Parser -> Parser: parse_mixin_arguments()
Parser -> Compiler: MixinCall 完成

Compiler -> Scope: lookup_mixin(name)
Scope --> Compiler: Vec<MixinDefinition>

alt 找到混合器定义
    Compiler -> Matcher: find_matching_mixin(definitions, call)
    
    loop 遍历每个混合器定义
        alt 有守卫条件
            Matcher -> GuardEval: evaluate_guard(guard, temp_scope)
            activate GuardEval
            
            GuardEval -> ParamBinder: bind_arguments_for_guard(params, args)
            activate ParamBinder
            ParamBinder --> GuardEval: 临时作用域
            deactivate ParamBinder
            
            GuardEval -> GuardEval: evaluate_expression(guard)
            GuardEval --> Matcher: 守卫求值结果 (bool)
            deactivate GuardEval
            
            alt 守卫条件为真
                Matcher --> Compiler: 匹配的混合器
            else 守卫条件为假
                note right: 继续检查下一个定义
            end
        else 无守卫条件
            Matcher --> Compiler: 第一个匹配的混合器
        end
    end
    
    alt 找到匹配的混合器
        Compiler -> ParamBinder: bind_mixin_arguments(mixin, call)
        activate ParamBinder
        
        ParamBinder -> ParamBinder: 创建新作用域
        
        loop 绑定每个参数
            alt 可变参数
                ParamBinder -> ParamBinder: collect_variadic_args(remaining_args)
                note right: 收集所有剩余参数到 @args...
            else 普通参数
                alt 提供了参数值
                    ParamBinder -> ParamBinder: bind_argument(param, arg_value)
                else 使用默认值
                    ParamBinder -> ParamBinder: bind_default_value(param)
                end
            end
        end
        
        ParamBinder --> Compiler: 参数绑定完成
        deactivate ParamBinder
        
        Compiler -> Compiler: push_scope(mixin_scope)
        
        loop 展开混合器体
            Compiler -> Compiler: compile_statement(body_stmt)
            note right: 在混合器作用域中执行
        end
        
        Compiler -> Compiler: pop_scope()
        
    else 没有匹配的混合器
        Compiler -> Compiler: 报告错误
        note right: UndefinedMixin 或 NoMatchingGuard
    end
    
else 未找到混合器定义
    Compiler -> Compiler: 报告 UndefinedMixin 错误
end

@enduml
```

---

## 3. 变量解析时序图

```plantuml
@startuml Variable_Resolution_Flow
!define RECTANGLE class

title 变量解析和插值处理流程

participant "编译器" as Compiler
participant "作用域栈" as ScopeStack
participant "表达式求值器" as Evaluator
participant "插值处理器" as Interpolator
participant "CSS 生成器" as Generator

note over Compiler, Generator: 变量声明阶段

Compiler -> Compiler: 检测到变量声明
note right: @color: red;

Compiler -> Evaluator: evaluate_expression(value)
activate Evaluator
Evaluator -> Evaluator: 处理表达式类型
alt 字面量
    Evaluator --> Compiler: 直接返回值
else 表达式计算
    Evaluator -> Evaluator: 递归求值
    Evaluator --> Compiler: 计算结果
end
deactivate Evaluator

Compiler -> ScopeStack: current_scope().define_variable(name, value)
ScopeStack -> ScopeStack: 存储到当前作用域

note over Compiler, Generator: 变量引用阶段

Compiler -> Compiler: 检测到变量引用
note right: color: @color;

Compiler -> ScopeStack: lookup_variable(name)
activate ScopeStack

ScopeStack -> ScopeStack: 在当前作用域查找
alt 在当前作用域找到
    ScopeStack --> Compiler: 变量值
else 在当前作用域未找到
    ScopeStack -> ScopeStack: 在父作用域查找
    alt 在父作用域找到
        ScopeStack --> Compiler: 变量值
    else 完全未找到
        ScopeStack --> Compiler: None
        Compiler -> Compiler: 报告 UndefinedVariable 错误
    end
end
deactivate ScopeStack

note over Compiler, Generator: 变量插值阶段

Compiler -> Compiler: 检测到插值语法
note right: .@{selector} { ... }

Compiler -> Interpolator: process_interpolation(template, type)
activate Interpolator

Interpolator -> Interpolator: parse_interpolation_parts(template)
note right: 分解为字面量和变量部分

loop 处理每个插值部分
    alt 字面量部分
        Interpolator -> Interpolator: 直接保留
    else 变量部分
        Interpolator -> ScopeStack: lookup_variable(var_name)
        ScopeStack --> Interpolator: 变量值
        
        Interpolator -> Interpolator: format_value_for_interpolation(value)
        note right: 根据上下文格式化\n(选择器/属性/值)
    end
end

Interpolator -> Interpolator: combine_parts()
Interpolator --> Compiler: 插值结果字符串
deactivate Interpolator

alt 选择器插值
    Compiler -> Generator: 使用插值后的选择器
else 属性插值
    Compiler -> Generator: 使用插值后的属性名
else 值插值
    Compiler -> Generator: 使用插值后的属性值
end

note over Compiler, Generator: 复杂表达式中的变量

Compiler -> Compiler: 检测到复杂表达式
note right: width: @base * 2 + 10px;

Compiler -> Evaluator: evaluate_expression(complex_expr)
activate Evaluator

Evaluator -> Evaluator: 识别二元运算
Evaluator -> Evaluator: evaluate_expression(left) // @base
Evaluator -> ScopeStack: lookup_variable("base")
ScopeStack --> Evaluator: base 变量值

Evaluator -> Evaluator: evaluate_expression(right) // 2
Evaluator -> Evaluator: perform_binary_operation(left_val, *, right_val)

Evaluator -> Evaluator: 继续处理 + 10px 部分
Evaluator --> Compiler: 最终计算结果
deactivate Evaluator

Compiler -> Generator: 输出计算后的值

@enduml
```

---

## 4. 错误处理时序图

```plantuml
@startuml Error_Handling_Flow
!define RECTANGLE class

title 错误检测和处理流程

participant "词法分析器" as Lexer
participant "语法分析器" as Parser  
participant "编译器" as Compiler
participant "错误报告器" as ErrorReporter
participant "用户界面" as UI

note over Lexer, UI: 词法错误处理

Lexer -> Lexer: 扫描输入字符
alt 遇到非法字符
    Lexer -> ErrorReporter: create_lex_error("Unexpected character", position)
    activate ErrorReporter
    ErrorReporter -> ErrorReporter: 构造 LexError
    ErrorReporter --> Lexer: LexError
    deactivate ErrorReporter
    Lexer --> Parser: Err(LexError)
else 字符串未闭合
    Lexer -> ErrorReporter: create_lex_error("Unterminated string", position)
    activate ErrorReporter
    ErrorReporter --> Lexer: LexError
    deactivate ErrorReporter
    Lexer --> Parser: Err(LexError)
end

note over Lexer, UI: 语法错误处理

Parser -> Parser: 解析 Token 流
alt 期望的 Token 不匹配
    Parser -> ErrorReporter: create_parse_error("Expected ';'", expected, found, position)
    activate ErrorReporter
    ErrorReporter -> ErrorReporter: 构造 ParseError
    ErrorReporter --> Parser: ParseError
    deactivate ErrorReporter
    
    Parser -> Parser: 尝试错误恢复
    alt 可以恢复
        Parser -> Parser: skip_to_next_statement()
        note right: 跳到下一个分号或大括号
        Parser -> Parser: 继续解析
    else 无法恢复
        Parser --> Compiler: Err(ParseError)
    end
    
else 混合器语法错误
    Parser -> ErrorReporter: create_parse_error("Invalid mixin syntax", position)
    activate ErrorReporter
    ErrorReporter --> Parser: ParseError
    deactivate ErrorReporter
    Parser --> Compiler: Err(ParseError)
end

note over Lexer, UI: 语义错误处理

Compiler -> Compiler: 编译 AST
alt 未定义变量
    Compiler -> Compiler: lookup_variable("undefined_var")
    Compiler -> ErrorReporter: undefined_variable_error("undefined_var", position)
    activate ErrorReporter
    ErrorReporter -> ErrorReporter: 构造 UndefinedVariableError
    ErrorReporter --> Compiler: UndefinedVariableError
    deactivate ErrorReporter
    
    Compiler -> Compiler: 记录错误但继续编译
    note right: 收集所有错误
    
else 未定义混合器
    Compiler -> Compiler: lookup_mixin("undefined_mixin")
    Compiler -> ErrorReporter: undefined_mixin_error("undefined_mixin", position)
    activate ErrorReporter
    ErrorReporter --> Compiler: UndefinedMixinError
    deactivate ErrorReporter
    
else 类型不匹配
    Compiler -> Compiler: 执行算术运算
    Compiler -> ErrorReporter: type_mismatch_error("number", "string", position)
    activate ErrorReporter
    ErrorReporter --> Compiler: TypeMismatchError
    deactivate ErrorReporter
    
else 除以零
    Compiler -> Compiler: 执行除法运算
    Compiler -> ErrorReporter: division_by_zero_error(position)
    activate ErrorReporter
    ErrorReporter --> Compiler: DivisionByZeroError
    deactivate ErrorReporter
end

note over Lexer, UI: 错误报告和展示

Compiler -> Compiler: 收集所有错误
alt 有错误存在
    Compiler -> ErrorReporter: format_error_report(errors)
    activate ErrorReporter
    
    loop 格式化每个错误
        ErrorReporter -> ErrorReporter: format_single_error(error)
        note right: 包含位置信息、上下文、建议
    end
    
    ErrorReporter --> Compiler: 格式化的错误报告
    deactivate ErrorReporter
    
    Compiler --> UI: Err(formatted_errors)
    
    UI -> UI: 显示错误信息
    note right: 高亮错误位置\n显示错误消息\n提供修复建议
    
else 无错误
    Compiler --> UI: Ok(compiled_css)
    UI -> UI: 显示编译结果
end

note over Lexer, UI: 错误恢复策略

alt 词法错误
    note right: 停止编译，无法恢复
else 语法错误  
    note right: 尝试同步到下一个语句边界
else 语义错误
    note right: 记录错误，继续编译收集更多错误
end

@enduml
```

---

## 5. 作用域管理时序图

```plantuml
@startuml Scope_Management_Flow
!define RECTANGLE class

title 作用域管理和变量生命周期

participant "编译器" as Compiler
participant "作用域栈" as ScopeStack
participant "全局作用域" as GlobalScope
participant "混合器作用域" as MixinScope
participant "块作用域" as BlockScope

note over Compiler, BlockScope: 编译开始 - 创建全局作用域

Compiler -> ScopeStack: 初始化作用域栈
ScopeStack -> GlobalScope: 创建全局作用域
GlobalScope -> GlobalScope: 初始化变量和混合器映射
ScopeStack -> ScopeStack: push(global_scope)

note over Compiler, BlockScope: 全局变量和混合器定义

Compiler -> Compiler: 编译全局变量声明
loop 处理每个全局变量
    Compiler -> ScopeStack: current_scope().define_variable(name, value)
    ScopeStack -> GlobalScope: 存储变量到全局作用域
end

Compiler -> Compiler: 编译混合器定义
loop 处理每个混合器定义
    Compiler -> ScopeStack: current_scope().define_mixin(name, definition)
    ScopeStack -> GlobalScope: 存储混合器到全局作用域
end

note over Compiler, BlockScope: 混合器调用 - 创建混合器作用域

Compiler -> Compiler: 检测到混合器调用
Compiler -> ScopeStack: lookup_mixin(name)
ScopeStack -> GlobalScope: 查找混合器定义
GlobalScope --> ScopeStack: 返回混合器定义
ScopeStack --> Compiler: 混合器定义

Compiler -> ScopeStack: 创建混合器作用域
ScopeStack -> MixinScope: 创建新作用域(parent: global_scope)
MixinScope -> MixinScope: 设置父作用域指针

Compiler -> Compiler: 绑定混合器参数
loop 绑定每个参数
    Compiler -> ScopeStack: current_scope().define_variable(param_name, arg_value)
    ScopeStack -> MixinScope: 存储参数到混合器作用域
end

ScopeStack -> ScopeStack: push(mixin_scope)

note over Compiler, BlockScope: 混合器内部编译

Compiler -> Compiler: 编译混合器体
loop 处理混合器内的语句
    alt 变量声明
        Compiler -> ScopeStack: current_scope().define_variable(name, value)
        ScopeStack -> MixinScope: 存储到混合器作用域
        note right: 局部变量，不影响全局
        
    else 变量引用
        Compiler -> ScopeStack: lookup_variable(name)
        ScopeStack -> MixinScope: 在混合器作用域查找
        
        alt 在混合器作用域找到
            MixinScope --> ScopeStack: 返回局部变量值
        else 在混合器作用域未找到
            ScopeStack -> GlobalScope: 在父作用域(全局)查找
            alt 在全局作用域找到
                GlobalScope --> ScopeStack: 返回全局变量值
            else 完全未找到
                ScopeStack --> Compiler: None (未定义变量错误)
            end
        end
        
    else 嵌套混合器调用
        Compiler -> Compiler: 递归处理混合器调用
        note right: 可能创建更深层的作用域
    end
end

note over Compiler, BlockScope: 混合器执行完毕 - 清理作用域

Compiler -> Compiler: 混合器体编译完成
ScopeStack -> ScopeStack: pop() // 移除混合器作用域
ScopeStack -> MixinScope: 销毁混合器作用域
note right: 混合器内的局部变量被清理

ScopeStack -> ScopeStack: 恢复到全局作用域

note over Compiler, BlockScope: 条件块作用域 (未来扩展)

alt 支持条件块 (if/when)
    Compiler -> Compiler: 检测到条件块
    ScopeStack -> BlockScope: 创建块作用域
    ScopeStack -> ScopeStack: push(block_scope)
    
    Compiler -> Compiler: 编译块内容
    note right: 块内变量不影响外部
    
    ScopeStack -> ScopeStack: pop() // 清理块作用域
end

note over Compiler, BlockScope: 作用域查找链示例

note right of ScopeStack
  变量查找优先级:
  1. 当前作用域 (混合器/块)
  2. 父作用域 (全局)
  3. 未找到 → 错误
  
  作用域栈结构:
  [混合器作用域] ← 当前
  [全局作用域]   ← 父级
  
  变量可见性:
  - 局部变量遮蔽全局变量
  - 参数变量优先级最高
  - 父作用域变量可访问
end note

@enduml
```

---

## 6. 守卫条件评估时序图

```plantuml
@startuml Guard_Evaluation_Flow
!define RECTANGLE class

title 混合器守卫条件评估流程

participant "编译器" as Compiler
participant "混合器匹配器" as Matcher
participant "守卫求值器" as GuardEvaluator
participant "表达式求值器" as ExprEvaluator
participant "临时作用域" as TempScope
participant "比较运算器" as Comparator

note over Compiler, Comparator: 守卫条件解析阶段 (在 Parser 中完成)

note right of Compiler
  示例混合器定义:
  .mixin(@size) when (@size > 10) { 
    font-size: large; 
  }
  .mixin(@size) when (@size <= 10) { 
    font-size: small; 
  }
end note

note over Compiler, Comparator: 混合器调用和守卫评估

Compiler -> Compiler: 检测到混合器调用
note right: .mixin(15);

Compiler -> Matcher: find_matching_mixin(mixin_definitions, call)
activate Matcher

loop 遍历所有同名混合器定义
    Matcher -> Matcher: 获取当前混合器定义
    
    alt 混合器有守卫条件
        Matcher -> GuardEvaluator: evaluate_guard(guard_expr, mixin_def, call_args)
        activate GuardEvaluator
        
        note over GuardEvaluator: 创建临时作用域用于守卫求值
        
        GuardEvaluator -> TempScope: 创建临时作用域
        activate TempScope
        TempScope -> TempScope: 初始化空的变量映射
        
        note over GuardEvaluator: 绑定调用参数到临时作用域
        
        loop 绑定每个参数
            GuardEvaluator -> GuardEvaluator: 获取参数名和值
            alt 可变参数
                GuardEvaluator -> GuardEvaluator: collect_variadic_arguments()
                GuardEvaluator -> TempScope: define_variable(param_name, collected_args)
            else 普通参数
                GuardEvaluator -> TempScope: define_variable(param_name, arg_value)
            end
        end
        
        note over GuardEvaluator: 在临时作用域中求值守卫表达式
        
        GuardEvaluator -> ExprEvaluator: evaluate_expression(guard_expr, temp_scope)
        activate ExprEvaluator
        
        alt 守卫表达式是比较运算
            ExprEvaluator -> ExprEvaluator: 识别二元比较运算
            note right: @size > 10
            
            ExprEvaluator -> ExprEvaluator: evaluate_expression(left_expr) // @size
            ExprEvaluator -> TempScope: lookup_variable("size")
            TempScope --> ExprEvaluator: 15 (参数值)
            
            ExprEvaluator -> ExprEvaluator: evaluate_expression(right_expr) // 10
            note right: 字面量，直接返回 10
            
            ExprEvaluator -> Comparator: compare(15, GreaterThan, 10)
            activate Comparator
            Comparator -> Comparator: 执行 15 > 10
            Comparator --> ExprEvaluator: true
            deactivate Comparator
            
            ExprEvaluator --> GuardEvaluator: Boolean(true)
            
        else 守卫表达式是复合条件
            note right: 例如: (@size > 10) and (@type = "large")
            ExprEvaluator -> ExprEvaluator: evaluate_logical_expression()
            ExprEvaluator --> GuardEvaluator: Boolean 结果
            
        else 守卫表达式是函数调用
            note right: 例如: when(defined(@optional))
            ExprEvaluator -> ExprEvaluator: evaluate_function_call()
            ExprEvaluator --> GuardEvaluator: Boolean 结果
        end
        
        deactivate ExprEvaluator
        
        GuardEvaluator -> TempScope: 清理临时作用域
        deactivate TempScope
        
        GuardEvaluator --> Matcher: 守卫评估结果 (true/false)
        deactivate GuardEvaluator
        
        alt 守卫条件为真
            Matcher -> Matcher: 找到匹配的混合器
            Matcher --> Compiler: 返回匹配的混合器定义
            note right: 停止搜索，使用此混合器
            
        else 守卫条件为假
            Matcher -> Matcher: 继续检查下一个混合器定义
            note right: 尝试下一个重载
        end
        
    else 混合器无守卫条件
        Matcher -> Matcher: 无条件匹配
        Matcher --> Compiler: 返回此混合器定义
        note right: 无守卫的混合器总是匹配
    end
end

alt 找到匹配的混合器
    note right of Matcher: 继续正常的混合器展开流程
else 没有找到匹配的混合器
    Matcher --> Compiler: None
    Compiler -> Compiler: 报告 "No matching guard" 错误
end

deactivate Matcher

note over Compiler, Comparator: 支持的守卫条件类型

note right of GuardEvaluator
  支持的比较运算符:
  - @size > 10     (大于)
  - @size < 5      (小于)  
  - @size >= 10    (大于等于)
  - @size <= 5     (小于等于)
  - @size = 10     (等于)
  - @size != 5     (不等于)
  
  支持的逻辑运算符:
  - (@a > 5) and (@b < 10)
  - (@a = 1) or (@b = 2)  
  - not (@a > 10)
  
  支持的函数:
  - defined(@var)  (变量是否定义)
  - type(@var)     (变量类型检查)
end note

@enduml
```

---

## 使用指南

### 如何阅读时序图

1. **纵向时间轴**: 从上到下表示时间流逝
2. **参与者**: 图顶部的各个组件/类
3. **消息**: 箭头表示方法调用或数据传递
4. **激活框**: 竖直的长方形表示对象处于活跃状态
5. **备注**: 提供额外的上下文信息

### 时序图说明

- **完整编译流程**: 展示从用户输入到 CSS 输出的完整过程
- **混合器处理**: 详细展示混合器定义、匹配和展开的复杂流程
- **变量解析**: 变量查找、插值处理的详细步骤
- **错误处理**: 各个阶段的错误检测和恢复机制
- **作用域管理**: 变量生命周期和作用域链管理
- **守卫条件评估**: 混合器守卫的复杂评估流程

这些时序图提供了系统动态行为的详细视图，帮助理解各组件如何协作完成复杂的编译任务。