//! LESS 的词法分析
//!
//! 此模块提供将 LESS 源代码标记化为可供解析器使用的标记流的功能。

use crate::ast::Position;
use crate::error::{Error, Result};
use std::fmt;

/// LESS 词法分析器识别的标记类型
///
/// 包含 LESS 语法中所有可能的标记类型，从字面量到运算符再到标点符号。
/// 词法分析器将源代码分解为这些标记的序列，供解析器使用。
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum TokenType {
    // 字面量
    String(String),
    Number(f64),
    Percentage(f64), // 50%
    Identifier(String),
    Hash(String),      // #color 或 #id
    AtKeyword(String), // @variable 或 @media
    Url(String),

    // 运算符
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
    Not,

    // 标点符号
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Colon,
    Comma,
    Dot,

    // 特殊符号
    Ampersand, // &
    Tilde,     // ~
    Pipe,      // |

    // Whitespace and comments
    Whitespace,
    Comment(String),

    // Variable interpolation
    VariableInterpolation(String), // @{variable}

    // Keywords
    When, // when (for mixin guards)

    // Variadic parameters
    Ellipsis, // ... for @param...

    // Special tokens
    Important,  // !important
    Default,    // !default
    MergeComma, // +: (property merge with comma)
    MergeSpace, // +_: (property merge with space)

    // End of file
    Eof,
}

/// 带位置信息的标记
///
/// 表示词法分析器识别的单个标记，包含标记类型、
/// 在源代码中的位置以及原始文本内容。
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// 标记的类型
    pub token_type: TokenType,
    /// 标记在源代码中的位置（行号和列号）
    pub position: Position,
    /// 标记的原始文本内容
    pub lexeme: String,
}

/// Lexer for LESS source code
pub struct Lexer {
    /// 原始输入字符串（保留用于调试和错误报告）
    #[allow(dead_code)]
    input: String,
    /// Character index (not byte index) - for proper Unicode support
    char_index: usize,
    /// Cached characters for efficient access
    chars: Vec<char>,
    line: usize,
    column: usize,
    current_char: Option<char>,
}

impl Lexer {
    /// Create a new lexer for the given input
    pub fn new(input: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.first().copied();
        Self {
            input,
            char_index: 0,
            chars,
            line: 1,
            column: 1,
            current_char,
        }
    }

    /// Get the current position
    pub fn current_position(&self) -> Position {
        Position::new(self.line, self.column)
    }

    /// Advance to the next character
    fn advance(&mut self) {
        if let Some(ch) = self.current_char {
            self.char_index += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        self.current_char = self.chars.get(self.char_index).copied();
    }

    /// Peek at the next character without consuming it
    fn peek(&self) -> Option<char> {
        self.chars.get(self.char_index + 1).copied()
    }

    /// Peek ahead by n characters without consuming them
    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.chars.get(self.char_index + n).copied()
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Read a string literal
    fn read_string(&mut self, quote: char) -> Result<String> {
        let mut value = String::new();
        self.advance(); // Skip opening quote

        while let Some(ch) = self.current_char {
            if ch == quote {
                self.advance(); // Skip closing quote
                return Ok(value);
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char {
                    match escaped {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '\'' => value.push('\''),
                        '"' => value.push('"'),
                        _ => {
                            value.push('\\');
                            value.push(escaped);
                        }
                    }
                    self.advance();
                }
            } else {
                value.push(ch);
                self.advance();
            }
        }

        Err(Error::lex_error(
            "Unterminated string literal",
            self.line,
            self.column,
        ))
    }

    /// Read a number (returns the number and whether it's a percentage)
    fn read_number(&mut self) -> (f64, bool) {
        let mut value = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() || ch == '.' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let number = value.parse().unwrap_or(0.0);

        // Check if this is followed by a percentage sign
        let is_percentage = if self.current_char == Some('%') {
            self.advance(); // consume the %
            true
        } else {
            false
        };

        (number, is_percentage)
    }

    /// Check if a character is valid for the start of an identifier
    fn is_identifier_start(ch: char) -> bool {
        ch.is_alphabetic() || ch == '_' || ch >= '\u{00A0}'
    }

    /// Check if a character is valid inside an identifier
    fn is_identifier_char(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_' || ch == '-' || ch >= '\u{00A0}'
    }

    /// Read an identifier
    fn read_identifier(&mut self) -> String {
        let mut value = String::new();

        while let Some(ch) = self.current_char {
            if Self::is_identifier_char(ch) {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        value
    }

    /// Read a hash token (#)
    fn read_hash(&mut self) -> String {
        let mut value = String::new();
        self.advance(); // Skip #

        while let Some(ch) = self.current_char {
            if Self::is_identifier_char(ch) {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        value
    }

    /// Read an at-keyword (@)
    fn read_at_keyword(&mut self) -> String {
        let mut value = String::new();
        self.advance(); // Skip @

        while let Some(ch) = self.current_char {
            if Self::is_identifier_char(ch) {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        value
    }

    /// Read variable interpolation (@{variable})
    fn read_variable_interpolation(&mut self) -> Result<String> {
        let mut value = String::new();
        self.advance(); // Skip @

        if self.current_char != Some('{') {
            return Err(Error::lex_error(
                "Expected '{' after '@' for variable interpolation",
                self.line,
                self.column,
            ));
        }

        self.advance(); // Skip {

        while let Some(ch) = self.current_char {
            if ch == '}' {
                self.advance(); // Skip }
                break;
            } else if Self::is_identifier_char(ch) {
                value.push(ch);
                self.advance();
            } else {
                return Err(Error::lex_error(
                    "Invalid character in variable interpolation",
                    self.line,
                    self.column,
                ));
            }
        }

        if value.is_empty() {
            return Err(Error::lex_error(
                "Empty variable interpolation",
                self.line,
                self.column,
            ));
        }

        Ok(value)
    }

    /// Read a comment (returns full comment including markers)
    fn read_comment(&mut self) -> Result<String> {
        let mut value = String::new();

        if self.current_char == Some('/') && self.peek() == Some('/') {
            // Line comment - include // marker
            value.push_str("//");
            self.advance(); // Skip first /
            self.advance(); // Skip second /

            while let Some(ch) = self.current_char {
                if ch == '\n' {
                    break;
                } else {
                    value.push(ch);
                    self.advance();
                }
            }
        } else if self.current_char == Some('/') && self.peek() == Some('*') {
            // Block comment - include /* */ markers
            let start_line = self.line;
            let start_col = self.column;
            value.push_str("/*");
            self.advance(); // Skip /
            self.advance(); // Skip *

            let mut found_end = false;
            while let Some(ch) = self.current_char {
                if ch == '*' && self.peek() == Some('/') {
                    self.advance(); // Skip *
                    self.advance(); // Skip /
                    value.push_str("*/");
                    found_end = true;
                    break;
                } else {
                    value.push(ch);
                    self.advance();
                }
            }

            if !found_end {
                return Err(Error::parse_error(
                    "Unterminated block comment",
                    start_line,
                    start_col,
                ));
            }
        }

        Ok(value)
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Result<Token> {
        loop {
            let position = self.current_position();

            match self.current_char {
                None => {
                    return Ok(Token {
                        token_type: TokenType::Eof,
                        position,
                        lexeme: String::new(),
                    });
                }

                Some(ch) if ch.is_whitespace() => {
                    self.skip_whitespace();
                    continue;
                }

                Some('"') | Some('\'') => {
                    let quote = self.current_char.unwrap();
                    let value = self.read_string(quote)?;
                    return Ok(Token {
                        token_type: TokenType::String(value.clone()),
                        position,
                        lexeme: format!("{}{}{}", quote, value, quote),
                    });
                }

                Some(ch) if ch.is_ascii_digit() => {
                    let (value, is_percentage) = self.read_number();
                    if is_percentage {
                        return Ok(Token {
                            token_type: TokenType::Percentage(value),
                            position,
                            lexeme: format!("{}%", value),
                        });
                    } else {
                        return Ok(Token {
                            token_type: TokenType::Number(value),
                            position,
                            lexeme: value.to_string(),
                        });
                    }
                }

                Some(ch) if Self::is_identifier_start(ch) => {
                    let value = self.read_identifier();
                    let lexeme = value.clone();

                    let token_type = match value.as_str() {
                        "and" => TokenType::And,
                        "or" => TokenType::Or,
                        "not" => TokenType::Not,
                        "when" => TokenType::When,
                        _ => TokenType::Identifier(value),
                    };

                    return Ok(Token {
                        token_type,
                        position,
                        lexeme,
                    });
                }

                Some('#') => {
                    let value = self.read_hash();
                    return Ok(Token {
                        token_type: TokenType::Hash(value.clone()),
                        position,
                        lexeme: format!("#{}", value),
                    });
                }

                Some('@') => {
                    // Check if this is variable interpolation @{...}
                    if self.peek() == Some('{') {
                        let value = self.read_variable_interpolation()?;
                        return Ok(Token {
                            token_type: TokenType::VariableInterpolation(value.clone()),
                            position,
                            lexeme: format!("@{{{}}}", value),
                        });
                    } else {
                        let value = self.read_at_keyword();
                        return Ok(Token {
                            token_type: TokenType::AtKeyword(value.clone()),
                            position,
                            lexeme: format!("@{}", value),
                        });
                    }
                }

                Some('/') if self.peek() == Some('/') || self.peek() == Some('*') => {
                    let value = self.read_comment()?;
                    return Ok(Token {
                        token_type: TokenType::Comment(value.clone()),
                        position,
                        lexeme: value,
                    });
                }

                Some('!') => {
                    self.advance();
                    let identifier = self.read_identifier();
                    let lexeme = format!("!{}", identifier);

                    let token_type = match identifier.as_str() {
                        "important" => TokenType::Important,
                        "default" => TokenType::Default,
                        _ => {
                            return Err(Error::lex_error(
                                format!("Unknown directive: !{identifier}"),
                                self.line,
                                self.column,
                            ))
                        }
                    };

                    return Ok(Token {
                        token_type,
                        position,
                        lexeme,
                    });
                }

                // Single character tokens
                Some('+') => {
                    // Check for +_: (merge with space) or +: (merge with comma)
                    if self.peek() == Some('_') && self.peek_ahead(2) == Some(':') {
                        self.advance(); // consume +
                        self.advance(); // consume _
                        self.advance(); // consume :
                        return Ok(Token {
                            token_type: TokenType::MergeSpace,
                            position,
                            lexeme: "+_:".to_string(),
                        });
                    } else if self.peek() == Some(':') {
                        self.advance(); // consume +
                        self.advance(); // consume :
                        return Ok(Token {
                            token_type: TokenType::MergeComma,
                            position,
                            lexeme: "+:".to_string(),
                        });
                    }
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::Plus,
                        position,
                        lexeme: "+".to_string(),
                    });
                }

                Some('-') => {
                    // Check if this is a CSS property name starting with - (like -webkit-border-radius)
                    if let Some(next_ch) = self.peek() {
                        if next_ch.is_alphabetic() {
                            // This is an identifier starting with -, read the whole thing
                            let value = self.read_identifier();
                            let lexeme = value.clone();

                            return Ok(Token {
                                token_type: TokenType::Identifier(value),
                                position,
                                lexeme,
                            });
                        }
                    }

                    // Otherwise it's just a minus operator
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::Minus,
                        position,
                        lexeme: "-".to_string(),
                    });
                }

                Some('*') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::Multiply,
                        position,
                        lexeme: "*".to_string(),
                    });
                }

                Some('/') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::Divide,
                        position,
                        lexeme: "/".to_string(),
                    });
                }

                Some('%') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::Modulo,
                        position,
                        lexeme: "%".to_string(),
                    });
                }

                Some('=') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::Equal,
                        position,
                        lexeme: "=".to_string(),
                    });
                }

                Some('<') => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token {
                            token_type: TokenType::LessThanOrEqual,
                            position,
                            lexeme: "<=".to_string(),
                        });
                    } else {
                        return Ok(Token {
                            token_type: TokenType::LessThan,
                            position,
                            lexeme: "<".to_string(),
                        });
                    }
                }

                Some('>') => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token {
                            token_type: TokenType::GreaterThanOrEqual,
                            position,
                            lexeme: ">=".to_string(),
                        });
                    } else {
                        return Ok(Token {
                            token_type: TokenType::GreaterThan,
                            position,
                            lexeme: ">".to_string(),
                        });
                    }
                }

                Some('(') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::LeftParen,
                        position,
                        lexeme: "(".to_string(),
                    });
                }

                Some(')') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::RightParen,
                        position,
                        lexeme: ")".to_string(),
                    });
                }

                Some('{') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::LeftBrace,
                        position,
                        lexeme: "{".to_string(),
                    });
                }

                Some('}') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::RightBrace,
                        position,
                        lexeme: "}".to_string(),
                    });
                }

                Some('[') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::LeftBracket,
                        position,
                        lexeme: "[".to_string(),
                    });
                }

                Some(']') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::RightBracket,
                        position,
                        lexeme: "]".to_string(),
                    });
                }

                Some(';') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::Semicolon,
                        position,
                        lexeme: ";".to_string(),
                    });
                }

                Some(':') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::Colon,
                        position,
                        lexeme: ":".to_string(),
                    });
                }

                Some(',') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::Comma,
                        position,
                        lexeme: ",".to_string(),
                    });
                }

                Some('.') => {
                    // Check for ellipsis (...)
                    if self.peek() == Some('.') && self.peek_ahead(2) == Some('.') {
                        self.advance(); // consume first dot
                        self.advance(); // consume second dot
                        self.advance(); // consume third dot
                        return Ok(Token {
                            token_type: TokenType::Ellipsis,
                            position,
                            lexeme: "...".to_string(),
                        });
                    } else {
                        self.advance(); // consume the dot
                        return Ok(Token {
                            token_type: TokenType::Dot,
                            position,
                            lexeme: ".".to_string(),
                        });
                    }
                }

                Some('&') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::Ampersand,
                        position,
                        lexeme: "&".to_string(),
                    });
                }

                Some('~') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::Tilde,
                        position,
                        lexeme: "~".to_string(),
                    });
                }

                Some('|') => {
                    self.advance();
                    return Ok(Token {
                        token_type: TokenType::Pipe,
                        position,
                        lexeme: "|".to_string(),
                    });
                }

                Some(ch) => {
                    return Err(Error::lex_error(
                        format!("Unexpected character: '{ch}'"),
                        self.line,
                        self.column,
                    ));
                }
            }
        }
    }

    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.token_type, TokenType::Eof);
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::String(_) => write!(f, "String"),
            TokenType::Number(_) => write!(f, "Number"),
            TokenType::Percentage(_) => write!(f, "Percentage"),
            TokenType::Identifier(_) => write!(f, "Identifier"),
            TokenType::Hash(_) => write!(f, "Hash"),
            TokenType::AtKeyword(_) => write!(f, "AtKeyword"),
            TokenType::Url(_) => write!(f, "Url"),
            TokenType::Plus => write!(f, "Plus"),
            TokenType::Minus => write!(f, "Minus"),
            TokenType::Multiply => write!(f, "Multiply"),
            TokenType::Divide => write!(f, "Divide"),
            TokenType::Modulo => write!(f, "Modulo"),
            TokenType::Equal => write!(f, "Equal"),
            TokenType::NotEqual => write!(f, "NotEqual"),
            TokenType::LessThan => write!(f, "LessThan"),
            TokenType::LessThanOrEqual => write!(f, "LessThanOrEqual"),
            TokenType::GreaterThan => write!(f, "GreaterThan"),
            TokenType::GreaterThanOrEqual => write!(f, "GreaterThanOrEqual"),
            TokenType::And => write!(f, "And"),
            TokenType::Or => write!(f, "Or"),
            TokenType::Not => write!(f, "Not"),
            TokenType::LeftParen => write!(f, "LeftParen"),
            TokenType::RightParen => write!(f, "RightParen"),
            TokenType::LeftBrace => write!(f, "LeftBrace"),
            TokenType::RightBrace => write!(f, "RightBrace"),
            TokenType::LeftBracket => write!(f, "LeftBracket"),
            TokenType::RightBracket => write!(f, "RightBracket"),
            TokenType::Semicolon => write!(f, "Semicolon"),
            TokenType::Colon => write!(f, "Colon"),
            TokenType::Comma => write!(f, "Comma"),
            TokenType::Dot => write!(f, "Dot"),
            TokenType::Ampersand => write!(f, "Ampersand"),
            TokenType::Tilde => write!(f, "Tilde"),
            TokenType::Pipe => write!(f, "Pipe"),
            TokenType::Whitespace => write!(f, "Whitespace"),
            TokenType::Comment(_) => write!(f, "Comment"),
            TokenType::VariableInterpolation(_) => write!(f, "VariableInterpolation"),
            TokenType::When => write!(f, "When"),
            TokenType::Ellipsis => write!(f, "Ellipsis"),
            TokenType::Important => write!(f, "Important"),
            TokenType::Default => write!(f, "Default"),
            TokenType::MergeComma => write!(f, "MergeComma"),
            TokenType::MergeSpace => write!(f, "MergeSpace"),
            TokenType::Eof => write!(f, "Eof"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("+ - * / %".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 6); // 5 operators + EOF
        assert!(matches!(tokens[0].token_type, TokenType::Plus));
        assert!(matches!(tokens[1].token_type, TokenType::Minus));
        assert!(matches!(tokens[2].token_type, TokenType::Multiply));
        assert!(matches!(tokens[3].token_type, TokenType::Divide));
        assert!(matches!(tokens[4].token_type, TokenType::Modulo));
        assert!(matches!(tokens[5].token_type, TokenType::Eof));
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("\"hello world\"".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 2); // string + EOF
        if let TokenType::String(s) = &tokens[0].token_type {
            assert_eq!(s, "hello world");
        } else {
            panic!("Expected string token");
        }
    }

    #[test]
    fn test_number() {
        let mut lexer = Lexer::new("123.45".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 2); // number + EOF
        if let TokenType::Number(n) = tokens[0].token_type {
            assert_eq!(n, 123.45);
        } else {
            panic!("Expected number token");
        }
    }

    #[test]
    fn test_at_keyword() {
        let mut lexer = Lexer::new("@variable @media".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3); // 2 at-keywords + EOF
        if let TokenType::AtKeyword(s) = &tokens[0].token_type {
            assert_eq!(s, "variable");
        } else {
            panic!("Expected at-keyword token");
        }
    }

    #[test]
    fn test_hash() {
        let mut lexer = Lexer::new("#ff0000".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 2); // hash + EOF
        if let TokenType::Hash(s) = &tokens[0].token_type {
            assert_eq!(s, "ff0000");
        } else {
            panic!("Expected hash token");
        }
    }

    #[test]
    fn test_comment() {
        let mut lexer = Lexer::new("// line comment".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 2); // comment + EOF
        if let TokenType::Comment(s) = &tokens[0].token_type {
            // Comment now includes the // marker
            assert_eq!(s, "// line comment");
        } else {
            panic!("Expected comment token");
        }

        // Test block comment
        let mut lexer2 = Lexer::new("/* block comment */".to_string());
        let tokens2 = lexer2.tokenize().unwrap();

        assert_eq!(tokens2.len(), 2); // comment + EOF
        if let TokenType::Comment(s) = &tokens2[0].token_type {
            // Comment now includes the /* */ markers
            assert_eq!(s, "/* block comment */");
        } else {
            panic!("Expected block comment token");
        }
    }
}
