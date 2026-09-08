//! LESS 的抽象语法树定义
//!
//! 此模块定义了表示 LESS 语法所需的所有 AST 节点，
//! 包括变量、混合器、选择器、属性和表达式。

use std::collections::HashMap;
use std::fmt;

pub mod expressions;
pub mod selectors;
pub mod values;

pub use expressions::*;
pub use selectors::*;
pub use values::*;

/// 用于错误报告的位置信息
/// Represents a position in the source code for error reporting
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

impl Position {
    /// Create a new position
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self { line: 1, column: 1 }
    }
}

/// LESS 样式表是规则和语句的集合
/// Root AST node representing a complete LESS stylesheet
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Stylesheet {
    /// All top-level statements in the stylesheet
    pub statements: Vec<Statement>,
    /// Position where this stylesheet starts
    pub position: Position,
}

impl Stylesheet {
    /// Create a new empty stylesheet
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a stylesheet with the given statements
    pub fn with_statements(statements: Vec<Statement>) -> Self {
        Self {
            statements,
            position: Position::default(),
        }
    }
}

/// LESS 文件中的顶级语句
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// 变量声明: @var: value;
    Variable(VariableDeclaration),
    /// 带选择器和声明的 CSS 规则
    Rule(Rule),
    /// CSS 声明: property: value;
    Declaration(Declaration),
    /// 混合器定义
    MixinDefinition(MixinDefinition),
    /// 混合器调用
    MixinCall(MixinCall),
    /// 导入语句
    Import(Import),
    /// At 规则（例如 @media、@keyframes）
    AtRule(AtRule),
    /// 注释
    Comment(Comment),
    /// 扩展语句
    Extend(Extend),
    /// each() 迭代调用
    EachCall(EachCall),
    /// Detached ruleset 调用: @var();
    DetachedRulesetCall(DetachedRulesetCall),
}

/// Detached ruleset 调用: @var()
#[derive(Debug, Clone, PartialEq)]
pub struct DetachedRulesetCall {
    /// 变量名（不含 @）
    pub name: String,
    /// Source position
    pub position: Position,
}

/// each() 迭代调用
#[derive(Debug, Clone, PartialEq)]
pub struct EachCall {
    /// 要迭代的列表表达式
    pub list: Expression,
    /// 模板块体
    pub body: Vec<Statement>,
    /// Source position
    pub position: Position,
}

/// 变量声明: @variable: value;
/// Variable declaration: `@name: value;`
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration {
    /// Variable name (without @)
    pub name: String,
    /// Variable value expression
    pub value: Expression,
    /// Whether this is a !default declaration
    pub default: bool, // 用于 !default 声明
    /// Source position
    pub position: Position,
}

impl VariableDeclaration {
    /// Create a new variable declaration
    pub fn new(name: String, value: Expression, position: Position) -> Self {
        Self {
            name,
            value,
            default: false,
            position,
        }
    }

    /// Mark this declaration as !default
    pub fn with_default(mut self) -> Self {
        self.default = true;
        self
    }
}

/// 带选择器和声明的 CSS 规则
/// CSS rule with selectors and declarations
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    /// Rule selectors
    pub selectors: Vec<Selector>,
    /// Property declarations
    pub declarations: Vec<Declaration>,
    /// Nested rules and statements
    pub nested_rules: Vec<Statement>,
    /// Source position
    pub position: Position,
}

impl Rule {
    /// Create a new rule with selectors
    pub fn new(selectors: Vec<Selector>, position: Position) -> Self {
        Self {
            selectors,
            declarations: Vec::new(),
            nested_rules: Vec::new(),
            position,
        }
    }

    /// Add declarations to this rule
    pub fn with_declarations(mut self, declarations: Vec<Declaration>) -> Self {
        self.declarations = declarations;
        self
    }

    /// Add nested rules to this rule
    pub fn with_nested_rules(mut self, nested_rules: Vec<Statement>) -> Self {
        self.nested_rules = nested_rules;
        self
    }
}

/// Property merge type for `+:` and `+_:` syntax
#[derive(Debug, Clone, PartialEq)]
pub enum MergeType {
    /// Merge with comma separator (`+:`)
    Comma,
    /// Merge with space separator (`+_:`)
    Space,
}

/// 属性声明: property: value;
/// CSS property declaration: `property: value;`
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    /// CSS property name
    pub property: String,
    /// Property value expression
    pub value: Expression,
    /// Whether !important is specified
    pub important: bool,
    /// Property merge type (None for normal, Some for +: or +_:)
    pub merge: Option<MergeType>,
    /// Source file override for source-map attribution (set when a declaration
    /// originates from a mixin/detached-ruleset defined in another file)
    pub source_file: Option<String>,
    /// Source position
    pub position: Position,
}

impl Declaration {
    /// Create a new declaration
    pub fn new(property: String, value: Expression, position: Position) -> Self {
        Self {
            property,
            value,
            important: false,
            merge: None,
            source_file: None,
            position,
        }
    }

    /// Mark this declaration as !important
    pub fn with_important(mut self) -> Self {
        self.important = true;
        self
    }

    /// Set merge type
    pub fn with_merge(mut self, merge_type: MergeType) -> Self {
        self.merge = Some(merge_type);
        self
    }
}

/// 混合器定义
/// Mixin definition: `.name(@param) { ... }`
#[derive(Debug, Clone, PartialEq)]
pub struct MixinDefinition {
    /// Mixin name
    pub name: String,
    /// Mixin parameters
    pub parameters: Vec<MixinParameter>,
    /// Guard condition
    pub guard: Option<Expression>,
    /// Mixin body statements
    pub body: Vec<Statement>,
    /// Source file path where the mixin is defined
    pub source_file: Option<String>,
    /// Source position
    pub position: Position,
}

impl MixinDefinition {
    /// Create a new mixin definition
    pub fn new(name: String, position: Position) -> Self {
        Self {
            name,
            parameters: Vec::new(),
            guard: None,
            body: Vec::new(),
            source_file: None,
            position,
        }
    }

    /// Add parameters to this mixin
    pub fn with_parameters(mut self, parameters: Vec<MixinParameter>) -> Self {
        self.parameters = parameters;
        self
    }

    /// Add guard condition to this mixin
    pub fn with_guard(mut self, guard: Expression) -> Self {
        self.guard = Some(guard);
        self
    }

    /// Add body statements to this mixin
    pub fn with_body(mut self, body: Vec<Statement>) -> Self {
        self.body = body;
        self
    }

    /// Attach the source file path where this mixin is defined
    pub fn with_source_file(mut self, source_file: impl Into<String>) -> Self {
        self.source_file = Some(source_file.into());
        self
    }
}

/// 混合器参数
/// Mixin parameter definition
#[derive(Debug, Clone, PartialEq)]
pub struct MixinParameter {
    /// Parameter name
    pub name: String,
    /// Default value if any
    pub default_value: Option<Expression>,
    /// Whether this is a variadic parameter (@rest...)
    pub variadic: bool, // 用于 @rest... 参数
    /// Pattern-match value (for `.mixin(dark)` style definitions)
    pub pattern_value: Option<Expression>,
    /// Source position
    pub position: Position,
}

impl MixinParameter {
    /// Create a new mixin parameter
    pub fn new(name: String, position: Position) -> Self {
        Self {
            name,
            default_value: None,
            variadic: false,
            pattern_value: None,
            position,
        }
    }

    /// Add default value to this parameter
    pub fn with_default(mut self, default: Expression) -> Self {
        self.default_value = Some(default);
        self
    }

    /// Mark this parameter as variadic
    pub fn variadic(mut self) -> Self {
        self.variadic = true;
        self
    }
}

/// 混合器调用
/// Mixin call: `.name(args);`
#[derive(Debug, Clone, PartialEq)]
pub struct MixinCall {
    /// Mixin name to call
    pub name: String,
    /// Arguments passed to the mixin
    pub arguments: Vec<Expression>,
    /// Whether !important is specified
    pub important: bool,
    /// Source position
    pub position: Position,
}

impl MixinCall {
    /// Create a new mixin call
    pub fn new(name: String, position: Position) -> Self {
        Self {
            name,
            arguments: Vec::new(),
            important: false,
            position,
        }
    }

    /// Add arguments to this mixin call
    pub fn with_arguments(mut self, arguments: Vec<Expression>) -> Self {
        self.arguments = arguments;
        self
    }

    /// Mark this mixin call as !important
    pub fn with_important(mut self) -> Self {
        self.important = true;
        self
    }
}

/// 导入语句
/// Import statement: `@import "file.less";`
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    /// Path to import
    pub path: String,
    /// Type of import
    pub import_type: ImportType,
    /// 可选导入 (@import (optional) ...) - 文件不存在时静默跳过
    pub optional: bool,
    /// 多次导入 (@import (multiple) ...) - 允许同一文件被多次引入
    pub multiple: bool,
    /// Media query condition
    pub media: Option<String>,
    /// Source position
    pub position: Position,
}

/// Type of import statement
///
/// `optional`/`multiple` 等正交选项见 [`Import::optional`] / [`Import::multiple`]，
/// 可与任意类型组合（less.js 允许 `@import (optional, reference) "file"` 这类写法）。
#[derive(Debug, Clone, PartialEq)]
pub enum ImportType {
    /// 作为 LESS 导入 (@import "file.less")，默认 once 语义
    Less,
    /// 作为 CSS 导入 (@import "file.css")
    Css,
    /// 内联导入 (@import (inline) "file.css")
    Inline,
    /// 引用导入 (@import (reference) "file.less")
    Reference,
}

impl Import {
    /// Create a new import statement
    pub fn new(path: String, import_type: ImportType, position: Position) -> Self {
        Self {
            path,
            import_type,
            optional: false,
            multiple: false,
            media: None,
            position,
        }
    }

    /// Mark this import as optional (silently skipped when the file is missing)
    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }

    /// Mark this import as allowing multiple inclusions of the same file
    pub fn with_multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// Add media query to this import
    pub fn with_media(mut self, media: String) -> Self {
        self.media = Some(media);
        self
    }
}

/// At 规则（例如 @media、@keyframes、@supports）
/// At-rule: `@media`, `@keyframes`, etc.
#[derive(Debug, Clone, PartialEq)]
pub struct AtRule {
    /// At-rule name (without @)
    pub name: String,
    /// Rule prelude/condition
    pub prelude: Option<String>,
    /// Rule block content
    pub block: Option<Vec<Statement>>,
    /// Source position
    pub position: Position,
}

impl AtRule {
    /// Create a new at-rule
    pub fn new(name: String, position: Position) -> Self {
        Self {
            name,
            prelude: None,
            block: None,
            position,
        }
    }

    /// Add prelude to this at-rule
    pub fn with_prelude(mut self, prelude: String) -> Self {
        self.prelude = Some(prelude);
        self
    }

    /// Add block content to this at-rule
    pub fn with_block(mut self, block: Vec<Statement>) -> Self {
        self.block = Some(block);
        self
    }
}

/// 注释节点
/// Comment in the source code
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    /// Comment text content
    pub content: String,
    /// Type of comment
    pub comment_type: CommentType,
    /// Source position
    pub position: Position,
}

/// Type of comment
#[derive(Debug, Clone, PartialEq)]
pub enum CommentType {
    /// 块注释 /* ... */
    Block,
    /// 行注释 // ...
    Line,
}

impl Comment {
    /// Create a new block comment
    pub fn block(content: String, position: Position) -> Self {
        Self {
            content,
            comment_type: CommentType::Block,
            position,
        }
    }

    /// Create a new line comment
    pub fn line(content: String, position: Position) -> Self {
        Self {
            content,
            comment_type: CommentType::Line,
            position,
        }
    }
}

/// Extension/Extend functionality
/// Extend directive: `:extend(.class)`
#[derive(Debug, Clone, PartialEq)]
pub struct Extend {
    /// Selectors to extend
    pub selectors: Vec<Selector>,
    /// Whether to extend all instances ("all" keyword)
    pub all: bool, // Simplified: if true, applies 'all' to all selectors (or just indicates presence)
    /// Source position
    pub position: Position,
}

impl Extend {
    /// Create a new extend directive
    pub fn new(selectors: Vec<Selector>, all: bool, position: Position) -> Self {
        Self {
            selectors,
            all,
            position,
        }
    }
}

/// Namespace access (e.g., #namespace > .mixin)
/// Namespace access: `namespace.member`
#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceAccess {
    /// Namespace name
    pub namespace: String,
    /// Member name
    pub member: String,
    /// Source position
    pub position: Position,
}

impl NamespaceAccess {
    /// Create a new namespace access
    pub fn new(namespace: String, member: String, position: Position) -> Self {
        Self {
            namespace,
            member,
            position,
        }
    }
}

/// Guard condition for mixins
/// Guard expression for conditional compilation
#[derive(Debug, Clone, PartialEq)]
pub struct Guard {
    /// Guard conditions
    pub conditions: Vec<GuardCondition>,
    /// Logical operator between conditions
    pub operator: GuardOperator,
    /// Source position
    pub position: Position,
}

/// Logical operator for guard expressions
#[derive(Debug, Clone, PartialEq)]
pub enum GuardOperator {
    /// Logical AND
    And,
    /// Logical OR
    Or,
    /// Logical NOT
    Not,
}

/// Individual guard condition
#[derive(Debug, Clone, PartialEq)]
pub struct GuardCondition {
    /// Left side of comparison
    pub left: Expression,
    /// Comparison operator
    pub operator: ComparisonOperator,
    /// Right side of comparison
    pub right: Expression,
    /// Source position
    pub position: Position,
}

/// Comparison operators for guard conditions
#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOperator {
    /// Equal (=)
    Equal,
    /// Not equal (!=)
    NotEqual,
    /// Less than (<)
    LessThan,
    /// Less than or equal comparison (<=)
    LessThanOrEqual,
    /// Greater than comparison (>)
    GreaterThan,
    /// Greater than or equal comparison (>=)
    GreaterThanOrEqual,
}

/// Maps (associative arrays) in LESS
#[derive(Debug, Clone, PartialEq)]
pub struct Map {
    /// The key-value pairs in the map
    pub entries: Vec<(Expression, Expression)>,
    /// Source position of this map expression
    pub position: Position,
}

impl Map {
    /// Create a new empty map at the given position
    pub fn new(position: Position) -> Self {
        Self {
            entries: Vec::new(),
            position,
        }
    }

    /// Add entries to this map
    pub fn with_entries(mut self, entries: Vec<(Expression, Expression)>) -> Self {
        self.entries = entries;
        self
    }
}

/// Variable scope for compilation
#[derive(Debug, Clone, Default)]
pub struct Scope {
    /// Variables defined in this scope
    pub variables: HashMap<String, Expression>,
    /// Mixins defined in this scope
    pub mixins: HashMap<String, Vec<MixinDefinition>>,
    /// Source file where each variable was defined (for source maps)
    pub variable_files: HashMap<String, String>,
    /// Parent scope for variable lookup
    pub parent: Option<Box<Scope>>,
}

impl Scope {
    /// Create a new empty scope
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new scope with the given parent scope
    pub fn with_parent(parent: Scope) -> Self {
        Self {
            variables: HashMap::new(),
            mixins: HashMap::new(),
            variable_files: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    /// Define a variable in this scope
    pub fn define_variable(&mut self, name: String, value: Expression) {
        self.variables.insert(name, value);
    }

    /// Record the source file where a variable was defined
    pub fn define_variable_file(&mut self, name: String, source_file: String) {
        self.variable_files.insert(name, source_file);
    }

    /// Look up the source file where a variable was defined
    pub fn lookup_variable_file(&self, name: &str) -> Option<&String> {
        self.variable_files.get(name).or_else(|| {
            self.parent
                .as_ref()
                .and_then(|p| p.lookup_variable_file(name))
        })
    }

    /// Define a mixin in this scope
    pub fn define_mixin(&mut self, name: String, mixin: MixinDefinition) {
        self.mixins.entry(name).or_default().push(mixin);
    }

    /// Look up a variable by name, searching parent scopes if needed
    pub fn lookup_variable(&self, name: &str) -> Option<&Expression> {
        self.variables.get(name).or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.lookup_variable(name))
        })
    }

    /// Look up a mixin by name, searching parent scopes if needed
    pub fn lookup_mixin(&self, name: &str) -> Option<&Vec<MixinDefinition>> {
        self.mixins.get(name).or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.lookup_mixin(name))
        })
    }
}

// Convenience traits for AST manipulation
/// Trait for AST nodes that can be visited by a visitor
pub trait Visitable {
    /// Accept a visitor and call the appropriate visit method
    fn accept<V: Visitor>(&self, visitor: &mut V);
}

/// Visitor pattern trait for traversing the AST
pub trait Visitor {
    /// Visit a stylesheet node
    fn visit_stylesheet(&mut self, _stylesheet: &Stylesheet) {}
    /// Visit a statement node
    fn visit_statement(&mut self, _statement: &Statement) {}
    /// Visit a variable declaration node
    fn visit_variable_declaration(&mut self, _var: &VariableDeclaration) {}
    /// Visit a CSS rule node
    fn visit_rule(&mut self, _rule: &Rule) {}
    /// Visit a CSS declaration node
    fn visit_declaration(&mut self, _decl: &Declaration) {}
    /// Visit a mixin definition node
    fn visit_mixin_definition(&mut self, _mixin: &MixinDefinition) {}
    /// Visit a mixin call node
    fn visit_mixin_call(&mut self, _call: &MixinCall) {}
    /// Visit an import statement node
    fn visit_import(&mut self, _import: &Import) {}
    /// Visit an at-rule node
    fn visit_at_rule(&mut self, _at_rule: &AtRule) {}
    /// Visit a comment node
    fn visit_comment(&mut self, _comment: &Comment) {}
    /// Visit an extend node
    fn visit_extend(&mut self, _extend: &Extend) {}
    /// Visit an expression node
    fn visit_expression(&mut self, _expr: &Expression) {}
    /// Visit a selector node
    fn visit_selector(&mut self, _selector: &Selector) {}
}

// Implement Visitable for major AST nodes
impl Visitable for Stylesheet {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_stylesheet(self);
        for statement in &self.statements {
            statement.accept(visitor);
        }
    }
}

impl Visitable for Statement {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        match self {
            Statement::Variable(var) => var.accept(visitor),
            Statement::Rule(rule) => rule.accept(visitor),
            Statement::Declaration(decl) => decl.accept(visitor),
            Statement::MixinDefinition(mixin) => mixin.accept(visitor),
            Statement::MixinCall(call) => call.accept(visitor),
            Statement::Import(import) => import.accept(visitor),
            Statement::AtRule(at_rule) => at_rule.accept(visitor),
            Statement::Comment(comment) => comment.accept(visitor),
            Statement::Extend(extend) => extend.accept(visitor),
            Statement::EachCall(_) => {} // each() is handled by compiler expansion
            Statement::DetachedRulesetCall(_) => {} // handled by compiler
        }
    }
}

impl Visitable for VariableDeclaration {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_variable_declaration(self);
        self.value.accept(visitor);
    }
}

impl Visitable for Rule {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_rule(self);
        for selector in &self.selectors {
            selector.accept(visitor);
        }
        for declaration in &self.declarations {
            declaration.accept(visitor);
        }
        for nested in &self.nested_rules {
            nested.accept(visitor);
        }
    }
}

impl Visitable for Declaration {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_declaration(self);
        self.value.accept(visitor);
    }
}

impl Visitable for MixinDefinition {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_mixin_definition(self);
        if let Some(guard) = &self.guard {
            guard.accept(visitor);
        }
        for statement in &self.body {
            statement.accept(visitor);
        }
    }
}

impl Visitable for MixinCall {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_mixin_call(self);
        for arg in &self.arguments {
            arg.accept(visitor);
        }
    }
}

impl Visitable for Import {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_import(self);
    }
}

impl Visitable for AtRule {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_at_rule(self);
        if let Some(block) = &self.block {
            for statement in block {
                statement.accept(visitor);
            }
        }
    }
}

impl Visitable for Comment {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_comment(self);
    }
}

impl Visitable for Extend {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_extend(self);
        for selector in &self.selectors {
            selector.accept(visitor);
        }
    }
}

// Display implementations for debugging
impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl fmt::Display for CommentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommentType::Block => write!(f, "block"),
            CommentType::Line => write!(f, "line"),
        }
    }
}

impl fmt::Display for ImportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportType::Less => write!(f, "less"),
            ImportType::Css => write!(f, "css"),
            ImportType::Inline => write!(f, "inline"),
            ImportType::Reference => write!(f, "reference"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(10, 5);
        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 5);
    }

    #[test]
    fn test_stylesheet_creation() {
        let stylesheet = Stylesheet::new();
        assert!(stylesheet.statements.is_empty());
        assert_eq!(stylesheet.position.line, 1);
        assert_eq!(stylesheet.position.column, 1);
    }

    #[test]
    fn test_variable_declaration() {
        let pos = Position::new(1, 1);
        let value = Expression::string("red".to_string(), pos.clone());
        let var = VariableDeclaration::new("color".to_string(), value, pos);

        assert_eq!(var.name, "color");
        assert!(!var.default);
    }

    #[test]
    fn test_variable_declaration_with_default() {
        let pos = Position::new(1, 1);
        let value = Expression::string("red".to_string(), pos.clone());
        let var = VariableDeclaration::new("color".to_string(), value, pos).with_default();

        assert!(var.default);
    }

    #[test]
    fn test_rule_creation() {
        let pos = Position::new(1, 1);
        let selector = Selector::simple(".test".to_string(), pos.clone());
        let rule = Rule::new(vec![selector], pos);

        assert_eq!(rule.selectors.len(), 1);
        assert!(rule.declarations.is_empty());
        assert!(rule.nested_rules.is_empty());
    }

    #[test]
    fn test_declaration_creation() {
        let pos = Position::new(1, 1);
        let value = Expression::string("red".to_string(), pos.clone());
        let decl = Declaration::new("color".to_string(), value, pos);

        assert_eq!(decl.property, "color");
        assert!(!decl.important);
    }

    #[test]
    fn test_declaration_with_important() {
        let pos = Position::new(1, 1);
        let value = Expression::string("red".to_string(), pos.clone());
        let decl = Declaration::new("color".to_string(), value, pos).with_important();

        assert!(decl.important);
    }

    #[test]
    fn test_mixin_definition() {
        let pos = Position::new(1, 1);
        let mixin = MixinDefinition::new("border-radius".to_string(), pos);

        assert_eq!(mixin.name, "border-radius");
        assert!(mixin.parameters.is_empty());
        assert!(mixin.guard.is_none());
        assert!(mixin.body.is_empty());
    }

    #[test]
    fn test_mixin_parameter() {
        let pos = Position::new(1, 1);
        let param = MixinParameter::new("radius".to_string(), pos);

        assert_eq!(param.name, "radius");
        assert!(param.default_value.is_none());
        assert!(!param.variadic);
    }

    #[test]
    fn test_mixin_parameter_variadic() {
        let pos = Position::new(1, 1);
        let param = MixinParameter::new("rest".to_string(), pos).variadic();

        assert!(param.variadic);
    }

    #[test]
    fn test_import_creation() {
        let pos = Position::new(1, 1);
        let import = Import::new("variables.less".to_string(), ImportType::Less, pos);

        assert_eq!(import.path, "variables.less");
        assert_eq!(import.import_type, ImportType::Less);
        assert!(import.media.is_none());
    }

    #[test]
    fn test_scope_operations() {
        let mut scope = Scope::new();
        let pos = Position::new(1, 1);
        let value = Expression::string("red".to_string(), pos);

        scope.define_variable("color".to_string(), value.clone());

        let found = scope.lookup_variable("color");
        assert!(found.is_some());
        assert_eq!(*found.unwrap(), value);

        let not_found = scope.lookup_variable("missing");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_scope_with_parent() {
        let mut parent = Scope::new();
        let pos = Position::new(1, 1);
        let parent_value = Expression::string("blue".to_string(), pos.clone());
        parent.define_variable("parent_color".to_string(), parent_value.clone());

        let mut child = Scope::with_parent(parent);
        let child_value = Expression::string("red".to_string(), pos);
        child.define_variable("child_color".to_string(), child_value.clone());

        // Child can access its own variables
        assert!(child.lookup_variable("child_color").is_some());
        // Child can access parent variables
        assert!(child.lookup_variable("parent_color").is_some());

        assert_eq!(*child.lookup_variable("child_color").unwrap(), child_value);
        assert_eq!(
            *child.lookup_variable("parent_color").unwrap(),
            parent_value
        );
    }
}
