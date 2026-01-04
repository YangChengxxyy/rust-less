//! LESS 编译器的错误处理
//!
//! 此模块定义了 LESS 编译期间可能发生的所有错误类型，
//! 包括解析错误、语义错误和运行时错误。

use std::fmt;

/// LESS 编译器操作的结果类型别名
pub type Result<T> = std::result::Result<T, Error>;

/// LESS 编译的全面错误类型
///
/// 每个变体都包含相关的上下文信息，如错误消息、位置信息等。
/// 使用 `message()` 方法获取用户友好的错误描述，
/// 使用 `line()` 和 `column()` 方法获取错误位置。
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum Error {
    /// 词法分析错误
    LexError {
        message: String,
        line: usize,
        column: usize,
    },

    /// 解析错误
    ParseError {
        message: String,
        line: usize,
        column: usize,
    },

    /// 语义分析错误
    SemanticError {
        message: String,
        line: usize,
        column: usize,
    },

    /// 未定义的变量引用
    UndefinedVariable {
        name: String,
        line: usize,
        column: usize,
    },

    /// 未定义的混合器引用
    UndefinedMixin {
        name: String,
        line: usize,
        column: usize,
    },

    /// 未定义的函数引用
    UndefinedFunction {
        name: String,
        line: usize,
        column: usize,
    },

    /// 操作中的类型不匹配
    TypeMismatch {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },

    /// 除以零
    DivisionByZero { line: usize, column: usize },

    /// 导入/包含错误
    ImportError {
        path: String,
        reason: String,
        line: usize,
        column: usize,
    },

    /// 导入中的循环依赖
    CircularDependency {
        path: String,
        line: usize,
        column: usize,
    },

    /// 文件 I/O 错误
    IoError {
        message: String,
        path: Option<String>,
    },

    /// 函数调用错误
    FunctionError {
        function: String,
        message: String,
        line: usize,
        column: usize,
    },

    /// 混合器参数错误
    MixinParameterError {
        mixin: String,
        expected: usize,
        found: usize,
        line: usize,
        column: usize,
    },

    /// 守卫条件评估错误
    GuardError {
        message: String,
        line: usize,
        column: usize,
    },

    /// 无限递归检测
    InfiniteRecursion {
        context: String,
        line: usize,
        column: usize,
    },

    /// 无效选择器错误
    InvalidSelector {
        selector: String,
        line: usize,
        column: usize,
    },

    /// 无效属性错误
    InvalidProperty {
        property: String,
        value: String,
        line: usize,
        column: usize,
    },

    /// 通用编译错误
    CompilationError { message: String },

    /// 内部编译器错误（bug）
    InternalError { message: String },
}

impl Error {
    /// 创建新的词法错误
    pub fn lex_error(message: impl Into<String>, line: usize, column: usize) -> Self {
        Error::LexError {
            message: message.into(),
            line,
            column,
        }
    }

    /// 创建新的解析错误
    pub fn parse_error(message: impl Into<String>, line: usize, column: usize) -> Self {
        Error::ParseError {
            message: message.into(),
            line,
            column,
        }
    }

    /// 创建新的语义错误
    pub fn semantic_error(message: impl Into<String>, line: usize, column: usize) -> Self {
        Error::SemanticError {
            message: message.into(),
            line,
            column,
        }
    }

    /// 创建未定义变量错误
    pub fn undefined_variable(name: impl Into<String>, line: usize, column: usize) -> Self {
        Error::UndefinedVariable {
            name: name.into(),
            line,
            column,
        }
    }

    /// 创建未定义混合器错误
    pub fn undefined_mixin(name: impl Into<String>, line: usize, column: usize) -> Self {
        Error::UndefinedMixin {
            name: name.into(),
            line,
            column,
        }
    }

    /// 创建未定义函数错误
    pub fn undefined_function(name: impl Into<String>, line: usize, column: usize) -> Self {
        Error::UndefinedFunction {
            name: name.into(),
            line,
            column,
        }
    }

    /// 创建类型不匹配错误
    pub fn type_mismatch(
        expected: impl Into<String>,
        found: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Error::TypeMismatch {
            expected: expected.into(),
            found: found.into(),
            line,
            column,
        }
    }

    /// 创建除以零错误
    pub fn division_by_zero(line: usize, column: usize) -> Self {
        Error::DivisionByZero { line, column }
    }

    /// 创建导入错误
    pub fn import_error(
        path: impl Into<String>,
        reason: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Error::ImportError {
            path: path.into(),
            reason: reason.into(),
            line,
            column,
        }
    }

    /// 创建循环依赖错误
    pub fn circular_dependency(path: impl Into<String>, line: usize, column: usize) -> Self {
        Error::CircularDependency {
            path: path.into(),
            line,
            column,
        }
    }

    /// 创建 I/O 错误
    pub fn io_error(message: impl Into<String>, path: Option<String>) -> Self {
        Error::IoError {
            message: message.into(),
            path,
        }
    }

    /// 创建函数错误
    pub fn function_error(
        function: impl Into<String>,
        message: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Error::FunctionError {
            function: function.into(),
            message: message.into(),
            line,
            column,
        }
    }

    /// 创建混合器参数错误
    pub fn mixin_parameter_error(
        mixin: impl Into<String>,
        expected: usize,
        found: usize,
        line: usize,
        column: usize,
    ) -> Self {
        Error::MixinParameterError {
            mixin: mixin.into(),
            expected,
            found,
            line,
            column,
        }
    }

    /// 创建守卫错误
    pub fn guard_error(message: impl Into<String>, line: usize, column: usize) -> Self {
        Error::GuardError {
            message: message.into(),
            line,
            column,
        }
    }

    /// 创建无限递归错误
    pub fn infinite_recursion(context: impl Into<String>, line: usize, column: usize) -> Self {
        Error::InfiniteRecursion {
            context: context.into(),
            line,
            column,
        }
    }

    /// 创建无效选择器错误
    pub fn invalid_selector(selector: impl Into<String>, line: usize, column: usize) -> Self {
        Error::InvalidSelector {
            selector: selector.into(),
            line,
            column,
        }
    }

    /// 创建无效属性错误
    pub fn invalid_property(
        property: impl Into<String>,
        value: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Error::InvalidProperty {
            property: property.into(),
            value: value.into(),
            line,
            column,
        }
    }

    /// 创建编译错误
    pub fn compilation_error(message: impl Into<String>) -> Self {
        Error::CompilationError {
            message: message.into(),
        }
    }

    /// 创建内部错误
    pub fn internal_error(message: impl Into<String>) -> Self {
        Error::InternalError {
            message: message.into(),
        }
    }

    /// 获取错误发生的行号（如果可用）
    pub fn line(&self) -> Option<usize> {
        match self {
            Error::LexError { line, .. }
            | Error::ParseError { line, .. }
            | Error::SemanticError { line, .. }
            | Error::UndefinedVariable { line, .. }
            | Error::UndefinedMixin { line, .. }
            | Error::UndefinedFunction { line, .. }
            | Error::TypeMismatch { line, .. }
            | Error::DivisionByZero { line, .. }
            | Error::ImportError { line, .. }
            | Error::CircularDependency { line, .. }
            | Error::FunctionError { line, .. }
            | Error::MixinParameterError { line, .. }
            | Error::GuardError { line, .. }
            | Error::InfiniteRecursion { line, .. }
            | Error::InvalidSelector { line, .. }
            | Error::InvalidProperty { line, .. } => Some(*line),
            Error::IoError { .. }
            | Error::CompilationError { .. }
            | Error::InternalError { .. } => None,
        }
    }

    /// 获取错误发生的列号（如果可用）
    pub fn column(&self) -> Option<usize> {
        match self {
            Error::LexError { column, .. }
            | Error::ParseError { column, .. }
            | Error::SemanticError { column, .. }
            | Error::UndefinedVariable { column, .. }
            | Error::UndefinedMixin { column, .. }
            | Error::UndefinedFunction { column, .. }
            | Error::TypeMismatch { column, .. }
            | Error::DivisionByZero { column, .. }
            | Error::ImportError { column, .. }
            | Error::CircularDependency { column, .. }
            | Error::FunctionError { column, .. }
            | Error::MixinParameterError { column, .. }
            | Error::GuardError { column, .. }
            | Error::InfiniteRecursion { column, .. }
            | Error::InvalidSelector { column, .. }
            | Error::InvalidProperty { column, .. } => Some(*column),
            Error::IoError { .. }
            | Error::CompilationError { .. }
            | Error::InternalError { .. } => None,
        }
    }

    /// 获取用户友好的错误信息
    pub fn message(&self) -> String {
        match self {
            Error::LexError { message, .. } => format!("Lexical error: {}", message),
            Error::ParseError { message, .. } => format!("Parse error: {}", message),
            Error::SemanticError { message, .. } => format!("Semantic error: {}", message),
            Error::UndefinedVariable { name, .. } => format!("Undefined variable: @{}", name),
            Error::UndefinedMixin { name, .. } => format!("Undefined mixin: .{}", name),
            Error::UndefinedFunction { name, .. } => format!("Undefined function: {}()", name),
            Error::TypeMismatch {
                expected, found, ..
            } => format!("Type mismatch: expected {}, found {}", expected, found),
            Error::DivisionByZero { .. } => "Division by zero".to_string(),
            Error::ImportError { path, reason, .. } => {
                format!("Import error '{}': {}", path, reason)
            }
            Error::CircularDependency { path, .. } => {
                format!("Circular dependency detected: {}", path)
            }
            Error::IoError { message, path } => match path {
                Some(p) => format!("I/O error in '{}': {}", p, message),
                None => format!("I/O error: {}", message),
            },
            Error::FunctionError {
                function, message, ..
            } => format!("Function '{}' error: {}", function, message),
            Error::MixinParameterError {
                mixin,
                expected,
                found,
                ..
            } => format!(
                "Mixin '{}' expects {} parameters, found {}",
                mixin, expected, found
            ),
            Error::GuardError { message, .. } => format!("Guard error: {}", message),
            Error::InfiniteRecursion { context, .. } => {
                format!("Infinite recursion detected in: {}", context)
            }
            Error::InvalidSelector { selector, .. } => {
                format!("Invalid selector: '{}'", selector)
            }
            Error::InvalidProperty {
                property, value, ..
            } => {
                format!("Invalid property '{}' with value '{}'", property, value)
            }
            Error::CompilationError { message } => format!("Compilation error: {}", message),
            Error::InternalError { message } => format!("Internal error: {}", message),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = self.message();
        if let (Some(line), Some(column)) = (self.line(), self.column()) {
            write!(f, "{} at line {}, column {}", message, line, column)
        } else {
            write!(f, "{}", message)
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::io_error(err.to_string(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::undefined_variable("color", 10, 5);
        assert_eq!(err.line(), Some(10));
        assert_eq!(err.column(), Some(5));
        assert!(err.message().contains("@color"));
    }

    #[test]
    fn test_error_display() {
        let err = Error::parse_error("Expected semicolon", 5, 10);
        let display = format!("{}", err);
        assert!(display.contains("Parse error"));
        assert!(display.contains("line 5"));
        assert!(display.contains("column 10"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::IoError { .. }));
    }

    #[test]
    fn test_error_without_position() {
        let err = Error::compilation_error("General error");
        assert_eq!(err.line(), None);
        assert_eq!(err.column(), None);
    }

    #[test]
    fn test_function_error() {
        let err = Error::function_error("lighten", "Invalid color value", 20, 15);
        assert_eq!(err.line(), Some(20));
        assert_eq!(err.column(), Some(15));
        assert!(err.message().contains("lighten"));
        assert!(err.message().contains("Invalid color value"));
    }

    #[test]
    fn test_mixin_parameter_error() {
        let err = Error::mixin_parameter_error("border-radius", 2, 1, 8, 12);
        assert!(err.message().contains("border-radius"));
        assert!(err.message().contains("expects 2"));
        assert!(err.message().contains("found 1"));
    }

    #[test]
    fn test_type_mismatch_error() {
        let err = Error::type_mismatch("number", "string", 15, 8);
        assert!(err.message().contains("expected number"));
        assert!(err.message().contains("found string"));
    }

    #[test]
    fn test_circular_dependency_error() {
        let err = Error::circular_dependency("styles.less", 1, 1);
        assert!(err.message().contains("Circular dependency"));
        assert!(err.message().contains("styles.less"));
    }
}
