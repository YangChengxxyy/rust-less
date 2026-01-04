//! LESS 语法解析器
//!
//! 此模块提供将 LESS 标记递归下降解析为 AST 的功能。

use crate::ast::*;
use crate::error::{Error, Result};
use crate::lexer::{Lexer, Token, TokenType};

/// LESS 语法解析器
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    /// 从词法分析器创建新的解析器
    pub fn new(mut lexer: Lexer) -> Result<Self> {
        let tokens = lexer.tokenize()?;
        Ok(Self { tokens, current: 0 })
    }

    /// 从输入字符串创建新的解析器
    pub fn from_string(input: String) -> Result<Self> {
        let lexer = Lexer::new(input);
        Self::new(lexer)
    }

    /// 解析整个样式表
    pub fn parse(&mut self) -> Result<Stylesheet> {
        let mut statements = Vec::new();
        let position = self.current_position();

        while !self.is_at_end() {
            if let Some(statement) = self.parse_statement()? {
                statements.push(statement);
            }
        }

        Ok(Stylesheet {
            statements,
            position,
        })
    }

    /// 获取当前标记
    fn current_token(&self) -> Token {
        self.tokens.get(self.current).cloned().unwrap_or(Token {
            token_type: TokenType::Eof,
            position: Position::default(),
            lexeme: String::new(),
        })
    }

    /// Get current position
    fn current_position(&self) -> Position {
        self.current_token().position
    }

    /// Check if we're at the end
    fn is_at_end(&self) -> bool {
        matches!(self.current_token().token_type, TokenType::Eof)
    }

    /// Advance to next token
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1).cloned().unwrap_or(Token {
            token_type: TokenType::Eof,
            position: Position::default(),
            lexeme: String::new(),
        })
    }

    /// Check if current token matches the given type
    fn check(&self, token_type: &TokenType) -> bool {
        std::mem::discriminant(&self.current_token().token_type)
            == std::mem::discriminant(token_type)
    }

    /// Consume a token if it matches the given type
    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(&token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume a token or return an error
    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            let pos = self.current_position();
            Err(Error::parse_error(message, pos.line, pos.column))
        }
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> Result<Option<Statement>> {
        // Skip whitespace
        while matches!(self.current_token().token_type, TokenType::Whitespace) {
            self.advance();
        }

        if self.is_at_end() {
            return Ok(None);
        }

        match &self.current_token().token_type {
            TokenType::Comment(content) => {
                let content = content.clone();
                let position = Position::new(
                    self.current_token().position.line,
                    self.current_token().position.column,
                );
                self.advance();

                let comment_type = if content.starts_with("/*") {
                    CommentType::Block
                } else {
                    CommentType::Line
                };

                // Extract content without comment markers
                let clean_content = if content.starts_with("/*") && content.ends_with("*/") {
                    content[2..content.len() - 2].to_string()
                } else if content.starts_with("//") {
                    content[2..].to_string()
                } else {
                    content
                };

                Ok(Some(Statement::Comment(Comment {
                    content: clean_content,
                    comment_type,
                    position,
                })))
            }
            TokenType::AtKeyword(keyword) => {
                // Check for @import first
                if keyword == "import" {
                    Ok(Some(Statement::Import(self.parse_import()?)))
                } else if self.is_variable_declaration() {
                    // Could be variable declaration or at-rule
                    Ok(Some(Statement::Variable(
                        self.parse_variable_declaration()?,
                    )))
                } else {
                    Ok(Some(Statement::AtRule(self.parse_at_rule()?)))
                }
            }
            _ => {
                // Check if this is a declaration first
                if self.is_declaration() {
                    Ok(Some(Statement::Declaration(self.parse_declaration()?)))
                } else if self.is_mixin_definition() {
                    Ok(Some(Statement::MixinDefinition(
                        self.parse_mixin_definition()?,
                    )))
                } else if self.is_mixin_call() {
                    Ok(Some(Statement::MixinCall(self.parse_mixin_call()?)))
                } else if self.is_extend_statement() {
                    Ok(Some(Statement::Extend(self.parse_extend()?)))
                } else {
                    // Try to parse as a CSS rule
                    Ok(Some(Statement::Rule(self.parse_rule()?)))
                }
            }
        }
    }

    /// Check if current position is an extend statement
    fn is_extend_statement(&self) -> bool {
        // &:extend(...)
        if !self.check(&TokenType::Ampersand) {
            return false;
        }

        let mut lookahead = self.current + 1;
        
        // Skip whitespace
        while lookahead < self.tokens.len() && matches!(self.tokens[lookahead].token_type, TokenType::Whitespace) {
            lookahead += 1;
        }

        if lookahead >= self.tokens.len() || self.tokens[lookahead].token_type != TokenType::Colon {
            return false;
        }
        lookahead += 1;

        // Skip whitespace
        while lookahead < self.tokens.len() && matches!(self.tokens[lookahead].token_type, TokenType::Whitespace) {
            lookahead += 1;
        }

        if lookahead >= self.tokens.len() {
            return false;
        }

        match &self.tokens[lookahead].token_type {
            TokenType::Identifier(s) if s == "extend" => true,
            _ => false,
        }
    }

    /// Parse an extend statement
    fn parse_extend(&mut self) -> Result<Extend> {
        let position = self.current_position();
        
        // Consume &
        self.consume(TokenType::Ampersand, "Expected '&'")?;
        
        // Skip whitespace
        while matches!(self.current_token().token_type, TokenType::Whitespace) {
            self.advance();
        }
        
        // Consume :
        self.consume(TokenType::Colon, "Expected ':'")?;
        
        // Skip whitespace
        while matches!(self.current_token().token_type, TokenType::Whitespace) {
            self.advance();
        }
        
        // Consume extend
        match &self.current_token().token_type {
            TokenType::Identifier(s) if s == "extend" => {
                self.advance();
            }
            _ => return Err(Error::parse_error("Expected 'extend'", position.line, position.column)),
        }
        
        // Skip whitespace
        while matches!(self.current_token().token_type, TokenType::Whitespace) {
            self.advance();
        }
        
        // Consume (
        self.consume(TokenType::LeftParen, "Expected '('")?;
        
        // Parse selectors
        let mut selectors = Vec::new();
        let mut all = false;
        
        loop {
            // Skip whitespace
            while matches!(self.current_token().token_type, TokenType::Whitespace) {
                self.advance();
            }
            
            if self.check(&TokenType::RightParen) {
                break;
            }
            
            // Parse selector
            let mut selector_str = String::new();
            let sel_pos = self.current_position();
            let mut current_is_all = false;
            
            while !self.is_at_end() && !self.check(&TokenType::Comma) && !self.check(&TokenType::RightParen) {
                // Check for "all" keyword if it's the last element
                if let TokenType::Identifier(s) = &self.current_token().token_type {
                    if s == "all" {
                         // Check if next is ) or comma
                         let mut next_idx = self.current + 1;
                         // Skip whitespace for check
                         while next_idx < self.tokens.len() && matches!(self.tokens[next_idx].token_type, TokenType::Whitespace) {
                             next_idx += 1;
                         }
                         
                         if next_idx < self.tokens.len() && (self.tokens[next_idx].token_type == TokenType::RightParen || self.tokens[next_idx].token_type == TokenType::Comma) {
                             current_is_all = true;
                             self.advance(); // consume 'all'
                             break;
                         }
                    }
                }
                
                if !matches!(self.current_token().token_type, TokenType::Whitespace) {
                     selector_str.push_str(&self.current_token().lexeme);
                } else if !selector_str.is_empty() {
                    selector_str.push(' ');
                }
                
                self.advance();
            }
            
            if !selector_str.is_empty() {
                let selector = Selector::simple(selector_str.trim().to_string(), sel_pos);
                selectors.push(selector);
            }
            
            if current_is_all {
                all = true;
            }
            
            // Skip whitespace
            while matches!(self.current_token().token_type, TokenType::Whitespace) {
                self.advance();
            }
            
            if self.match_token(TokenType::Comma) {
                continue;
            } else {
                break;
            }
        }
        
        self.consume(TokenType::RightParen, "Expected ')'")?;
        
        // Optional semicolon
        // Skip whitespace
        while matches!(self.current_token().token_type, TokenType::Whitespace) {
            self.advance();
        }
        self.match_token(TokenType::Semicolon);
        
        Ok(Extend::new(selectors, all, position))
    }

    /// Check if current position is a variable declaration
    fn is_variable_declaration(&self) -> bool {
        // Look ahead to see if there's a colon after the at-keyword
        let mut lookahead = self.current + 1;
        while lookahead < self.tokens.len() {
            match &self.tokens[lookahead].token_type {
                TokenType::Colon => return true,
                TokenType::Whitespace => {
                    lookahead += 1;
                    continue;
                }
                _ => return false,
            }
        }
        false
    }

    /// Parse a variable declaration
    fn parse_variable_declaration(&mut self) -> Result<VariableDeclaration> {
        let position = self.current_position();

        // Parse @variable-name
        let name = if let TokenType::AtKeyword(var_name) = &self.current_token().token_type {
            let name = var_name.clone();
            self.advance();
            name
        } else {
            return Err(Error::parse_error(
                "Expected variable name",
                position.line,
                position.column,
            ));
        };

        // Skip whitespace
        while matches!(self.current_token().token_type, TokenType::Whitespace) {
            self.advance();
        }

        self.consume(TokenType::Colon, "Expected ':' after variable name")?;

        let value = self.parse_expression()?;

        // Check for !default
        let default = self.match_token(TokenType::Default);

        self.match_token(TokenType::Semicolon); // Optional semicolon

        Ok(VariableDeclaration {
            name,
            value,
            default,
            position,
        })
    }

    /// Parse an @import statement
    fn parse_import(&mut self) -> Result<Import> {
        use crate::ast::Import;

        let position = self.current_position();

        // Consume @import keyword
        self.advance();

        // Skip whitespace
        while matches!(self.current_token().token_type, TokenType::Whitespace) {
            self.advance();
        }

        // Parse the import path (string or url)
        let path = match &self.current_token().token_type {
            TokenType::String(s) => {
                let path = s.clone();
                self.advance();
                path
            }
            TokenType::Identifier(s) if s == "url" => {
                // Handle url() syntax
                self.advance();
                self.consume(TokenType::LeftParen, "Expected '(' after url")?;

                // Skip whitespace
                while matches!(self.current_token().token_type, TokenType::Whitespace) {
                    self.advance();
                }

                let url = match &self.current_token().token_type {
                    TokenType::String(s) => {
                        let url = s.clone();
                        self.advance();
                        url
                    }
                    _ => {
                        // Collect tokens until )
                        let mut url = String::new();
                        while !self.is_at_end() && !self.check(&TokenType::RightParen) {
                            url.push_str(&self.current_token().lexeme);
                            self.advance();
                        }
                        url
                    }
                };

                self.consume(TokenType::RightParen, "Expected ')' after url")?;
                url
            }
            _ => {
                // Collect path as identifier sequence
                let mut path = String::new();
                while !self.is_at_end()
                    && !matches!(
                        self.current_token().token_type,
                        TokenType::Semicolon | TokenType::Whitespace
                    )
                {
                    path.push_str(&self.current_token().lexeme);
                    self.advance();
                }
                path
            }
        };

        // Skip whitespace
        while matches!(self.current_token().token_type, TokenType::Whitespace) {
            self.advance();
        }

        // Check for media query
        let media = if !self.is_at_end() && !self.check(&TokenType::Semicolon) {
            let mut media_str = String::new();
            while !self.is_at_end() && !self.check(&TokenType::Semicolon) {
                if !media_str.is_empty()
                    && !matches!(self.current_token().token_type, TokenType::Whitespace)
                {
                    media_str.push(' ');
                }
                if !matches!(self.current_token().token_type, TokenType::Whitespace) {
                    media_str.push_str(&self.current_token().lexeme);
                }
                self.advance();
            }
            if media_str.is_empty() {
                None
            } else {
                Some(media_str)
            }
        } else {
            None
        };

        // Consume optional semicolon
        self.match_token(TokenType::Semicolon);

        // Determine import type based on path extension
        let import_type = if path.ends_with(".css") {
            ImportType::Css
        } else {
            ImportType::Less
        };

        let mut import = Import::new(path, import_type, position);
        if let Some(m) = media {
            import = import.with_media(m);
        }

        Ok(import)
    }

    /// Parse an at-rule
    fn parse_at_rule(&mut self) -> Result<AtRule> {
        let position = self.current_position();

        let name = if let TokenType::AtKeyword(rule_name) = &self.current_token().token_type {
            let name = rule_name.clone();
            self.advance();
            name
        } else {
            return Err(Error::parse_error(
                "Expected at-rule name",
                position.line,
                position.column,
            ));
        };

        // Parse prelude (everything until { or ;)
        let mut prelude = String::new();
        let mut need_space = false;
        while !self.is_at_end()
            && !matches!(
                self.current_token().token_type,
                TokenType::LeftBrace | TokenType::Semicolon
            )
        {
            // Skip whitespace tokens but preserve spaces in the prelude
            if matches!(self.current_token().token_type, TokenType::Whitespace) {
                need_space = true;
                self.advance();
            } else {
                // Add space before token if needed
                if need_space && !prelude.is_empty() {
                    prelude.push(' ');
                }

                // Add the token
                prelude.push_str(&self.current_token().lexeme);

                // Add space after colon for media queries
                if matches!(self.current_token().token_type, TokenType::Colon) {
                    prelude.push(' ');
                }

                need_space = false;
                self.advance();
            }
        }

        let mut at_rule = AtRule::new(name, position);
        if !prelude.trim().is_empty() {
            at_rule = at_rule.with_prelude(prelude.trim().to_string());
        }

        // Check if this is a block rule or statement rule
        if self.match_token(TokenType::LeftBrace) {
            let mut statements = Vec::new();
            while !self.is_at_end() && !self.check(&TokenType::RightBrace) {
                // Skip whitespace
                while matches!(self.current_token().token_type, TokenType::Whitespace) {
                    self.advance();
                }

                if self.is_at_end() || self.check(&TokenType::RightBrace) {
                    break;
                }

                if let Some(statement) = self.parse_statement()? {
                    statements.push(statement);
                }
            }
            self.consume(TokenType::RightBrace, "Expected '}' after at-rule body")?;
            at_rule = at_rule.with_block(statements);
        } else {
            self.match_token(TokenType::Semicolon);
        }

        Ok(at_rule)
    }

    /// Parse a rule
    fn parse_rule(&mut self) -> Result<Rule> {
        let position = self.current_position();
        let selectors = self.parse_selectors()?;

        self.consume(TokenType::LeftBrace, "Expected '{' after selectors")?;

        let mut declarations = Vec::new();
        let mut nested_rules = Vec::new();

        while !self.is_at_end() && !self.check(&TokenType::RightBrace) {
            // Skip whitespace
            while matches!(self.current_token().token_type, TokenType::Whitespace) {
                self.advance();
            }

            if self.is_at_end() || self.check(&TokenType::RightBrace) {
                break;
            }

            // Try to parse as declaration first
            if self.is_declaration() {
                declarations.push(self.parse_declaration()?);
            } else {
                // Parse as nested rule or variable declaration
                if let Some(statement) = self.parse_statement()? {
                    nested_rules.push(statement);
                }
            }
        }

        self.consume(TokenType::RightBrace, "Expected '}' after rule body")?;

        Ok(Rule::new(selectors, position)
            .with_declarations(declarations)
            .with_nested_rules(nested_rules))
    }

    /// Check if current position is a declaration
    fn is_declaration(&self) -> bool {
        // Simple heuristic: look for property: value pattern
        let mut lookahead = self.current;

        // Skip over potential property name
        if matches!(
            self.tokens.get(lookahead).map(|t| &t.token_type),
            Some(TokenType::Identifier(_))
        ) {
            lookahead += 1;

            // Skip whitespace
            while matches!(
                self.tokens.get(lookahead).map(|t| &t.token_type),
                Some(TokenType::Whitespace)
            ) {
                lookahead += 1;
            }

            // Check for colon
            matches!(
                self.tokens.get(lookahead).map(|t| &t.token_type),
                Some(TokenType::Colon)
            )
        } else {
            false
        }
    }

    /// Parse a list of selectors
    pub fn parse_selectors(&mut self) -> Result<Vec<Selector>> {
        let mut selectors = Vec::new();
        let position = self.current_position();

        // Build selector parts
        let mut parts: Vec<SelectorPart> = Vec::new();

        while !self.is_at_end() && !self.check(&TokenType::LeftBrace) {
            match &self.current_token().token_type {
                TokenType::VariableInterpolation(var_name) => {
                    // Add interpolation as a simple selector
                    let interp = SimpleSelector::Interpolation {
                        variable: var_name.clone(),
                        position: self.current_position(),
                    };
                    
                    // Add to last part if open (no combinator), or start new part
                    if let Some(last) = parts.last_mut() {
                        if last.combinator.is_none() {
                            last.simple_selectors.push(interp);
                        } else {
                            parts.push(SelectorPart {
                                simple_selectors: vec![interp],
                                combinator: None,
                                position: self.current_position(),
                            });
                        }
                    } else {
                        parts.push(SelectorPart {
                            simple_selectors: vec![interp],
                            combinator: None,
                            position: self.current_position(),
                        });
                    }
                    self.advance();
                }
                TokenType::Whitespace => {
                    self.advance();
                }
                _ => {
                    // Check if this is an @keyword that should end selector parsing
                    if matches!(self.current_token().token_type, TokenType::AtKeyword(_)) {
                        break;
                    }

                    // For other tokens, build a simple text-based selector
                    let mut selector_text = String::new();
                    let part_position = self.current_position();
                    let mut paren_depth = 0;
                    let mut last_token_end: Option<(usize, usize)> = None;

                    while !self.is_at_end()
                        && !self.check(&TokenType::LeftBrace)
                        && !matches!(
                            self.current_token().token_type,
                            TokenType::VariableInterpolation(_)
                        )
                        && !matches!(self.current_token().token_type, TokenType::AtKeyword(_))
                    {
                        let token = self.current_token();
                        
                        if matches!(token.token_type, TokenType::Whitespace) {
                            if paren_depth == 0 {
                                break;
                            }
                        }
                        
                        // Check for implicit space (since Lexer might skip whitespace)
                        if let Some((last_line, last_col_end)) = last_token_end {
                            if token.position.line > last_line || token.position.column > last_col_end {
                                selector_text.push(' ');
                            }
                        }
                        
                        if token.token_type == TokenType::LeftParen {
                            paren_depth += 1;
                        } else if token.token_type == TokenType::RightParen {
                            if paren_depth > 0 {
                                paren_depth -= 1;
                            }
                        }

                        selector_text.push_str(&token.lexeme);
                        
                        let len = token.lexeme.chars().count();
                        last_token_end = Some((token.position.line, token.position.column + len));
                        
                        self.advance();
                    }

                    if !selector_text.is_empty() {
                        // Parse the selector text into appropriate SelectorPart types
                        let mut parsed_parts = self.parse_selector_parts_from_text(
                            &selector_text,
                            part_position.clone(),
                        )?;
                        parts.append(&mut parsed_parts);
                    }
                }
            }
        }

        if !parts.is_empty() {
            let selector = Selector {
                parts,
                position,
            };
            selectors.push(selector);
        }

        Ok(selectors)
    }

    /// Parse selector parts from text
    fn parse_selector_parts_from_text(
        &self,
        text: &str,
        position: Position,
    ) -> Result<Vec<SelectorPart>> {
        let mut parts = Vec::new();
        let mut current_simple_selectors = Vec::new();
        let mut idx = 0;
        let bytes = text.as_bytes();
        let len = text.len();

        while idx < len {
            let start_char = bytes[idx] as char;
            
            if start_char == ' ' {
                // Space implies Descendant combinator
                if !current_simple_selectors.is_empty() {
                    parts.push(SelectorPart {
                        simple_selectors: current_simple_selectors.clone(),
                        combinator: Some(Combinator::Descendant),
                        position: position.clone(),
                    });
                    current_simple_selectors.clear();
                }
                idx += 1;
                continue;
            }

            if start_char == '&' {
                 current_simple_selectors.push(SimpleSelector::Parent(position.clone()));
                 idx += 1;
                 // Check if followed by suffix (e.g. &-large)
                 if idx < len {
                     let c = bytes[idx] as char;
                     if c != '.' && c != '#' && c != ':' && c != '[' && c != ' ' {
                         // It's a suffix, treat as Type
                         let (name, next_idx) = self.consume_selector_name(text, idx);
                         idx = next_idx;
                         current_simple_selectors.push(SimpleSelector::Type {
                             name,
                             position: position.clone(),
                         });
                     }
                 }
            } else if start_char == '.' {
                idx += 1;
                let (name, next_idx) = self.consume_selector_name(text, idx);
                idx = next_idx;
                current_simple_selectors.push(SimpleSelector::Class {
                    name,
                    position: position.clone(),
                });
            } else if start_char == '#' {
                idx += 1;
                let (name, next_idx) = self.consume_selector_name(text, idx);
                idx = next_idx;
                current_simple_selectors.push(SimpleSelector::Id {
                    name,
                    position: position.clone(),
                });
            } else if start_char == ':' {
                idx += 1;
                let is_element = if idx < len && bytes[idx] as char == ':' {
                    idx += 1;
                    true
                } else {
                    false
                };
                
                let (name, next_idx) = self.consume_selector_name(text, idx);
                idx = next_idx;
                
                let mut argument = None;
                if idx < len && bytes[idx] as char == '(' {
                    let (arg_str, next_idx) = self.consume_parentheses(text, idx);
                    argument = Some(arg_str);
                    idx = next_idx;
                }
                
                if is_element {
                     current_simple_selectors.push(SimpleSelector::PseudoElement {
                        name,
                        position: position.clone(),
                    });
                } else {
                     current_simple_selectors.push(SimpleSelector::PseudoClass {
                        name,
                        argument,
                        position: position.clone(),
                    });
                }
            } else if start_char == '[' {
                 let (content, next_idx) = self.consume_brackets(text, idx);
                 idx = next_idx;
                 let (name, op, val) = self.parse_attribute_content(&content);
                 current_simple_selectors.push(SimpleSelector::Attribute {
                     name,
                     operator: op,
                     value: val,
                     case_insensitive: false, // TODO: support 'i' flag
                     position: position.clone(),
                 });
            } else if start_char == '*' {
                current_simple_selectors.push(SimpleSelector::Universal(position.clone()));
                idx += 1;
            } else {
                // Type selector
                let (name, next_idx) = self.consume_selector_name(text, idx);
                if !name.is_empty() {
                    current_simple_selectors.push(SimpleSelector::Type {
                        name,
                        position: position.clone(),
                    });
                    idx = next_idx;
                } else {
                    idx += 1; // Safety
                }
            }
        }
        
        if !current_simple_selectors.is_empty() {
            parts.push(SelectorPart {
                simple_selectors: current_simple_selectors,
                combinator: None,
                position: position.clone(),
            });
        }
        
        Ok(parts)
    }

    fn consume_selector_name(&self, text: &str, start: usize) -> (String, usize) {
        let mut idx = start;
        let bytes = text.as_bytes();
        let len = text.len();
        while idx < len {
             let c = bytes[idx] as char;
             if c == '.' || c == '#' || c == ':' || c == '[' || c == ' ' || c == '(' || c == ')' {
                 break;
             }
             idx += 1;
        }
        (text[start..idx].to_string(), idx)
    }

    fn consume_parentheses(&self, text: &str, start: usize) -> (String, usize) {
        let mut idx = start;
        let bytes = text.as_bytes();
        let len = text.len();
        let mut depth;
        
        if idx < len && bytes[idx] as char == '(' {
            depth = 1;
            idx += 1;
        } else {
             return (String::new(), idx);
        }
        
        let content_start = idx;
        while idx < len && depth > 0 {
             let c = bytes[idx] as char;
             if c == '(' {
                 depth += 1;
             } else if c == ')' {
                 depth -= 1;
             }
             if depth > 0 {
                 idx += 1;
             }
        }
        
        let content = text[content_start..idx].to_string();
        if idx < len {
             idx += 1;
        }
        (content, idx)
    }

    fn consume_brackets(&self, text: &str, start: usize) -> (String, usize) {
        let mut idx = start;
        let bytes = text.as_bytes();
        let len = text.len();
        
        if idx < len && bytes[idx] as char == '[' {
            idx += 1;
        } else {
             return (String::new(), idx);
        }
        
        let content_start = idx;
        while idx < len {
             let c = bytes[idx] as char;
             if c == ']' {
                 break;
             }
             idx += 1;
        }
        
        let content = text[content_start..idx].to_string();
        if idx < len {
             idx += 1;
        }
        (content, idx)
    }

    fn parse_attribute_content(&self, content: &str) -> (String, Option<AttributeOperator>, Option<String>) {
        let operators = ["=", "~=", "|=", "^=", "$=", "*="];
        for op_str in operators.iter() {
            if let Some(pos) = content.find(op_str) {
                let name = content[..pos].trim().to_string();
                let value_part = content[pos + op_str.len()..].trim();
                let value = if (value_part.starts_with('"') && value_part.ends_with('"')) || 
                             (value_part.starts_with('\'') && value_part.ends_with('\'')) {
                    if value_part.len() >= 2 {
                        value_part[1..value_part.len()-1].to_string()
                    } else {
                        value_part.to_string()
                    }
                } else {
                    value_part.to_string()
                };
                
                let op = match *op_str {
                    "=" => AttributeOperator::Equal,
                    "~=" => AttributeOperator::Includes,
                    "|=" => AttributeOperator::DashMatch,
                    "^=" => AttributeOperator::Prefix,
                    "$=" => AttributeOperator::Suffix,
                    "*=" => AttributeOperator::Substring,
                    _ => AttributeOperator::Equal,
                };
                return (name, Some(op), Some(value));
            }
        }
        (content.trim().to_string(), None, None)
    }

    /// Check if current position is a mixin definition
    fn is_mixin_definition(&self) -> bool {
        // Mixin definition: .name(@param) { ... } or .name() { ... } or .name { ... }
        // Can have optional parentheses
        let start_token = self.current_token();
        let mut lookahead = self.current + 1;

        if matches!(start_token.token_type, TokenType::Dot) {
            // Skip whitespace
            while lookahead < self.tokens.len() {
                match &self.tokens[lookahead].token_type {
                    TokenType::Whitespace => {
                        lookahead += 1;
                        continue;
                    }
                    TokenType::Identifier(_) => break,
                    _ => return false,
                }
            }

            if lookahead >= self.tokens.len() {
                return false;
            }

            // Must be followed by an identifier
            if !matches!(self.tokens[lookahead].token_type, TokenType::Identifier(_)) {
                return false;
            }

            lookahead += 1;
        } else if matches!(start_token.token_type, TokenType::Hash(_)) {
            // Hash token already includes the identifier
        } else {
            return false;
        }

        // Skip whitespace
        while lookahead < self.tokens.len() {
            match &self.tokens[lookahead].token_type {
                TokenType::Whitespace => {
                    lookahead += 1;
                    continue;
                }
                TokenType::LeftParen => {
                    // Has parentheses - check if it ends with { to be a definition
                    lookahead += 1;
                    let mut paren_depth = 1;
                    while lookahead < self.tokens.len() && paren_depth > 0 {
                        match &self.tokens[lookahead].token_type {
                            TokenType::LeftParen => paren_depth += 1,
                            TokenType::RightParen => paren_depth -= 1,
                            _ => {}
                        }
                        lookahead += 1;
                    }
                    // Skip whitespace after )
                    while lookahead < self.tokens.len() {
                        match &self.tokens[lookahead].token_type {
                            TokenType::Whitespace => lookahead += 1,
                            TokenType::When => {
                                // Handle when clause - skip to after the guard expression
                                lookahead += 1;
                                // Skip whitespace
                                while lookahead < self.tokens.len()
                                    && matches!(
                                        self.tokens[lookahead].token_type,
                                        TokenType::Whitespace
                                    )
                                {
                                    lookahead += 1;
                                }
                                // Skip guard expression in parentheses
                                if lookahead < self.tokens.len()
                                    && matches!(
                                        self.tokens[lookahead].token_type,
                                        TokenType::LeftParen
                                    )
                                {
                                    lookahead += 1;
                                    let mut guard_paren_depth = 1;
                                    while lookahead < self.tokens.len() && guard_paren_depth > 0 {
                                        match &self.tokens[lookahead].token_type {
                                            TokenType::LeftParen => guard_paren_depth += 1,
                                            TokenType::RightParen => guard_paren_depth -= 1,
                                            _ => {}
                                        }
                                        lookahead += 1;
                                    }
                                    // After guard, skip whitespace and look for {
                                    while lookahead < self.tokens.len() {
                                        match &self.tokens[lookahead].token_type {
                                            TokenType::Whitespace => lookahead += 1,
                                            TokenType::LeftBrace => return true,
                                            _ => return false,
                                        }
                                    }
                                }
                                return false;
                            }
                            TokenType::LeftBrace => return true,
                            _ => return false,
                        }
                    }

                    return false;
                }
                TokenType::LeftBrace => {
                    // No parentheses, just { - this is likely a CSS rule, not a mixin definition
                    // Only treat as mixin if we're in a context where mixins are expected
                    // For now, be conservative and return false
                    return false;
                }
                _ => return false,
            }
        }

        false
    }

    /// Check if current position is a mixin call
    fn is_mixin_call(&self) -> bool {
        // Mixin call: .name; or .name(); or #name; or #name();
        // Also supports namespaces: #ns > .mixin();
        // But NOT if it's followed by { which would be a definition or CSS rule
        let mut lookahead = self.current;
        
        // Loop to consume parts of the path (e.g., #ns > .mixin)
        loop {
            // Check start of part
            let start_token = if lookahead < self.tokens.len() {
                &self.tokens[lookahead]
            } else {
                return false;
            };

            if matches!(start_token.token_type, TokenType::Dot) {
                lookahead += 1;
                // Skip whitespace
                while lookahead < self.tokens.len() {
                    match &self.tokens[lookahead].token_type {
                        TokenType::Whitespace => {
                            lookahead += 1;
                            continue;
                        }
                        TokenType::Identifier(_) => break,
                        _ => return false,
                    }
                }

                if lookahead >= self.tokens.len() {
                    return false;
                }

                // Must be followed by an identifier
                if !matches!(self.tokens[lookahead].token_type, TokenType::Identifier(_)) {
                    return false;
                }

                lookahead += 1;
            } else if matches!(start_token.token_type, TokenType::Hash(_)) {
                // Hash token already includes the identifier
                lookahead += 1;
            } else {
                return false;
            }

            // Check for next part separator (>, space) or end of path
            let mut next_token_idx = lookahead;
            let mut has_whitespace = false;
            
            // Skip whitespace
            while next_token_idx < self.tokens.len() && matches!(self.tokens[next_token_idx].token_type, TokenType::Whitespace) {
                has_whitespace = true;
                next_token_idx += 1;
            }
            
            if next_token_idx >= self.tokens.len() {
                return false;
            }
            
            match &self.tokens[next_token_idx].token_type {
                TokenType::GreaterThan => {
                    // Continue to next part
                    lookahead = next_token_idx + 1;
                    // Skip whitespace after >
                    while lookahead < self.tokens.len() && matches!(self.tokens[lookahead].token_type, TokenType::Whitespace) {
                        lookahead += 1;
                    }
                    continue;
                }
                TokenType::Dot | TokenType::Hash(_) => {
                    if has_whitespace {
                        // Space separator (descendant)
                        lookahead = next_token_idx;
                        continue;
                    } else {
                        // .foo.bar - chained selector. Mixin names usually don't have this but selectors do.
                        // Less treats .foo.bar() as calling a mixin named .foo.bar?
                        // No, mixin names are simple.
                        // But #ns.mixin is valid?
                        // For now, let's assume if no whitespace and next is dot/hash, it's part of selector.
                        // But mixin calls usually single class/id.
                        // Unless we support #ns.mixin() which is technically valid in CSS selector syntax.
                        // Let's assume yes.
                        lookahead = next_token_idx;
                        continue;
                    }
                }
                TokenType::LeftParen => {
                    lookahead = next_token_idx;
                    break; // End of path, start of arguments
                }
                TokenType::Semicolon => {
                    return true; // End of call
                }
                TokenType::RightBrace => {
                    return true; // End of block, implicit end of call
                }
                TokenType::Important => {
                    // !important after mixin call
                     lookahead = next_token_idx;
                     break;
                }
                TokenType::LeftBrace => {
                    return false; // This is a rule/definition
                }
                _ => {
                    // Unknown token. 
                    // If we just had a path like #ns .mixin, and now we see something else.
                    // If it's a mixin call, it should end with ; or be followed by !important or ).
                    // If we are here, it means we didn't find ( or ; or !.
                    // So it's likely a selector for a rule.
                    return false;
                }
            }
        }

        // We broke out of loop at ( or !important
        // Skip whitespace
        while lookahead < self.tokens.len() {
            match &self.tokens[lookahead].token_type {
                TokenType::Whitespace => {
                    lookahead += 1;
                    continue;
                }
                TokenType::LeftParen => {
                    // Has parentheses - check what comes after )
                    lookahead += 1;
                    let mut paren_depth = 1;
                    while lookahead < self.tokens.len() && paren_depth > 0 {
                        match &self.tokens[lookahead].token_type {
                            TokenType::LeftParen => paren_depth += 1,
                            TokenType::RightParen => paren_depth -= 1,
                            _ => {}
                        }
                        lookahead += 1;
                    }
                    // Skip whitespace after )
                    while lookahead < self.tokens.len() {
                        match &self.tokens[lookahead].token_type {
                            TokenType::Whitespace => lookahead += 1,
                            TokenType::LeftBrace => return false, // This is a definition
                            TokenType::Semicolon => return true,  // This is a call
                            TokenType::RightBrace => return true, // End of block, this is a call
                            TokenType::Important => {
                                lookahead += 1;
                                continue;
                            }
                            _ => return true, // Assume it's a call followed by other content
                        }
                    }

                    return true; // End of file after ), assume call
                }
                TokenType::Important => {
                     lookahead += 1;
                     // Expect ; or }
                     while lookahead < self.tokens.len() {
                        match &self.tokens[lookahead].token_type {
                            TokenType::Whitespace => lookahead += 1,
                            TokenType::Semicolon => return true,
                            TokenType::RightBrace => return true,
                             _ => return false, // !important followed by something else
                        }
                     }
                     return true;
                }
                TokenType::Semicolon => return true,  // .mixin;
                TokenType::LeftBrace => return false, // .mixin { ... } is not a call
                _ => return false,
            }
        }

        false
    }

    /// Parse a mixin definition
    fn parse_mixin_definition(&mut self) -> Result<MixinDefinition> {
        let position = self.current_position();
        let mut name = String::new();

        if self.match_token(TokenType::Dot) {
            name.push('.');
            // Skip whitespace
            while matches!(self.current_token().token_type, TokenType::Whitespace) {
                self.advance();
            }

            // Parse mixin name
            if let TokenType::Identifier(id) = &self.current_token().token_type {
                name.push_str(id);
                self.advance();
            } else {
                return Err(Error::parse_error(
                    "Expected mixin name",
                    position.line,
                    position.column,
                ));
            }
        } else if let TokenType::Hash(val) = &self.current_token().token_type {
            let val = val.clone();
            name.push('#');
            name.push_str(&val);
            self.advance();
        } else {
            return Err(Error::parse_error(
                "Expected '.' or '#' for mixin definition",
                position.line,
                position.column,
            ));
        }

        // Skip whitespace
        while matches!(self.current_token().token_type, TokenType::Whitespace) {
            self.advance();
        }

        let mut mixin = MixinDefinition::new(name, position);

        // Parse parameters if present
        if self.match_token(TokenType::LeftParen) {
            let parameters = self.parse_mixin_parameters()?;
            mixin = mixin.with_parameters(parameters);
            self.consume(TokenType::RightParen, "Expected ')' after mixin parameters")?;
        }

        // Skip whitespace
        while matches!(self.current_token().token_type, TokenType::Whitespace) {
            self.advance();
        }

        // Parse guard clause if present
        if self.match_token(TokenType::When) {
            // Skip whitespace after 'when'
            while matches!(self.current_token().token_type, TokenType::Whitespace) {
                self.advance();
            }

            let guard = self.parse_guard_expression()?;
            mixin = mixin.with_guard(guard);

            // Skip whitespace after guard
            while matches!(self.current_token().token_type, TokenType::Whitespace) {
                self.advance();
            }
        }

        // Parse body
        self.consume(TokenType::LeftBrace, "Expected '{' for mixin body")?;

        let mut body = Vec::new();
        while !self.is_at_end() && !self.check(&TokenType::RightBrace) {
            // Skip whitespace
            while matches!(self.current_token().token_type, TokenType::Whitespace) {
                self.advance();
            }

            if self.is_at_end() || self.check(&TokenType::RightBrace) {
                break;
            }

            if let Some(statement) = self.parse_statement()? {
                body.push(statement);
            }
        }

        self.consume(TokenType::RightBrace, "Expected '}' after mixin body")?;

        mixin = mixin.with_body(body);
        Ok(mixin)
    }

    /// Parse mixin parameters
    fn parse_mixin_parameters(&mut self) -> Result<Vec<MixinParameter>> {
        let mut parameters = Vec::new();

        while !self.is_at_end() && !self.check(&TokenType::RightParen) {
            // Skip whitespace
            while matches!(self.current_token().token_type, TokenType::Whitespace) {
                self.advance();
            }

            if self.check(&TokenType::RightParen) {
                break;
            }

            let position = self.current_position();

            // Parse parameter name (should start with @)
            let name = if let TokenType::AtKeyword(param_name) = &self.current_token().token_type {
                let name = param_name.clone();
                self.advance();
                name
            } else {
                return Err(Error::parse_error(
                    "Expected parameter name starting with @",
                    position.line,
                    position.column,
                ));
            };

            let mut parameter = MixinParameter::new(name, position);

            // Skip whitespace
            while matches!(self.current_token().token_type, TokenType::Whitespace) {
                self.advance();
            }

            // Check for ellipsis (variadic parameter)
            if self.match_token(TokenType::Ellipsis) {
                parameter = parameter.variadic();
            }

            // Skip whitespace
            while matches!(self.current_token().token_type, TokenType::Whitespace) {
                self.advance();
            }

            // Check for default value (only for non-variadic parameters)
            if !parameter.variadic && self.match_token(TokenType::Colon) {
                let default_value = self.parse_expression()?;
                parameter = parameter.with_default(default_value);
            }

            parameters.push(parameter);

            // Skip whitespace
            while matches!(self.current_token().token_type, TokenType::Whitespace) {
                self.advance();
            }

            // Check for comma
            if self.match_token(TokenType::Comma) {
                // Skip whitespace after comma
                while matches!(self.current_token().token_type, TokenType::Whitespace) {
                    self.advance();
                }
            } else if !self.check(&TokenType::RightParen) {
                return Err(Error::parse_error(
                    "Expected ',' or ')' in parameter list",
                    self.current_position().line,
                    self.current_position().column,
                ));
            }
        }

        Ok(parameters)
    }

    /// Parse a mixin call
    fn parse_mixin_call(&mut self) -> Result<MixinCall> {
        let position = self.current_position();
        let mut name = String::new();

        loop {
            // Check if it starts with . or #
            if self.match_token(TokenType::Dot) {
                name.push('.');
                // Skip whitespace
                while matches!(self.current_token().token_type, TokenType::Whitespace) {
                    self.advance();
                }

                // Parse mixin name
                if let TokenType::Identifier(id) = &self.current_token().token_type {
                    name.push_str(id);
                    self.advance();
                } else {
                    return Err(Error::parse_error(
                        "Expected mixin name",
                        position.line,
                        position.column,
                    ));
                }
            } else if let TokenType::Hash(val) = &self.current_token().token_type {
                let val = val.clone();
                name.push('#');
                name.push_str(&val);
                self.advance();
            } else {
                return Err(Error::parse_error(
                    "Expected '.' or '#' for mixin call",
                    position.line,
                    position.column,
                ));
            }

            // Check for descendant combinator '>' or whitespace acting as descendant
            let mut lookahead = self.current;
            let mut has_whitespace = false;
            
            // Skip whitespace
            while lookahead < self.tokens.len() && matches!(self.tokens[lookahead].token_type, TokenType::Whitespace) {
                has_whitespace = true;
                lookahead += 1;
            }

            if lookahead < self.tokens.len() {
                match &self.tokens[lookahead].token_type {
                    TokenType::GreaterThan => {
                        // Found '>', consume it and continue
                        self.current = lookahead + 1;
                        name.push_str(" > ");
                        
                        // Skip whitespace after >
                        while matches!(self.current_token().token_type, TokenType::Whitespace) {
                            self.advance();
                        }
                        continue;
                    }
                    TokenType::Dot | TokenType::Hash(_) => {
                        // Found another selector part without >, but with whitespace
                        if has_whitespace {
                            self.current = lookahead;
                            name.push(' ');
                            continue;
                        }
                    }
                    _ => {}
                }
            }
            
            break;
        }

        let mut mixin_call = MixinCall::new(name, position);

        // Skip whitespace
        while matches!(self.current_token().token_type, TokenType::Whitespace) {
            self.advance();
        }

        // Parse arguments if present
        if self.match_token(TokenType::LeftParen) {
            let arguments = self.parse_mixin_arguments()?;
            mixin_call = mixin_call.with_arguments(arguments);
            self.consume(TokenType::RightParen, "Expected ')' after mixin arguments")?;
        }

        // Check for !important
        if self.match_token(TokenType::Important) {
            mixin_call = mixin_call.with_important();
        }

        // Consume semicolon
        self.match_token(TokenType::Semicolon);

        Ok(mixin_call)
    }

    /// Parse mixin arguments
    fn parse_mixin_arguments(&mut self) -> Result<Vec<expressions::Expression>> {
        let mut arguments = Vec::new();

        while !self.is_at_end() && !self.check(&TokenType::RightParen) {
            // Skip whitespace
            while matches!(self.current_token().token_type, TokenType::Whitespace) {
                self.advance();
            }

            if self.check(&TokenType::RightParen) {
                break;
            }

            let argument = self.parse_expression()?;
            arguments.push(argument);

            // Skip whitespace
            while matches!(self.current_token().token_type, TokenType::Whitespace) {
                self.advance();
            }

            // Check for comma
            if self.match_token(TokenType::Comma) {
                // Skip whitespace after comma
                while matches!(self.current_token().token_type, TokenType::Whitespace) {
                    self.advance();
                }
            } else if !self.check(&TokenType::RightParen) {
                return Err(Error::parse_error(
                    "Expected ',' or ')' in argument list",
                    self.current_position().line,
                    self.current_position().column,
                ));
            }
        }

        Ok(arguments)
    }

    /// Parse a guard expression for mixin guards
    fn parse_guard_expression(&mut self) -> Result<expressions::Expression> {
        // For now, we'll implement basic guard parsing
        // Guards are typically parenthesized expressions like (@param > 10)

        self.consume(
            TokenType::LeftParen,
            "Expected '(' to start guard expression",
        )?;

        // Skip whitespace
        while matches!(self.current_token().token_type, TokenType::Whitespace) {
            self.advance();
        }

        let guard_expr = self.parse_expression()?;

        // Skip whitespace
        while matches!(self.current_token().token_type, TokenType::Whitespace) {
            self.advance();
        }

        self.consume(
            TokenType::RightParen,
            "Expected ')' to end guard expression",
        )?;

        Ok(guard_expr)
    }

    /// Parse a declaration
    fn parse_declaration(&mut self) -> Result<Declaration> {
        let position = self.current_position();

        let property = if let TokenType::Identifier(prop) = &self.current_token().token_type {
            let name = prop.clone();
            self.advance();
            name
        } else {
            return Err(Error::parse_error(
                "Expected property name",
                position.line,
                position.column,
            ));
        };

        // Skip whitespace
        while matches!(self.current_token().token_type, TokenType::Whitespace) {
            self.advance();
        }

        self.consume(TokenType::Colon, "Expected ':' after property name")?;

        let value = self.parse_declaration_value()?;

        let important = if self.match_token(TokenType::Important) {
            true
        } else {
            false
        };

        self.match_token(TokenType::Semicolon); // Optional semicolon

        Ok(Declaration {
            property,
            value,
            important,
            position,
        })
    }

    /// Parse a declaration value (supports space-separated values)
    fn parse_declaration_value(&mut self) -> Result<Expression> {
        let mut values = Vec::new();
        let position = self.current_position();

        // Skip leading whitespace
        while matches!(self.current_token().token_type, TokenType::Whitespace) {
            self.advance();
        }

        // Parse first expression
        if !self.is_at_end()
            && !self.check(&TokenType::Semicolon)
            && !self.check(&TokenType::Important)
            && !self.check(&TokenType::RightBrace)
        {
            values.push(self.parse_expression()?);
        }

        // Parse additional space-separated values
        loop {
            // Skip whitespace
            while matches!(self.current_token().token_type, TokenType::Whitespace) {
                self.advance();
            }

            // Stop if we hit a terminator
            if self.is_at_end()
                || self.check(&TokenType::Semicolon)
                || self.check(&TokenType::Important)
                || self.check(&TokenType::RightBrace)
            {
                break;
            }

            // Try to parse another expression
            let next_expr = self.parse_expression()?;
            values.push(next_expr);
        }

        // Return single expression or list
        if values.len() == 1 {
            Ok(values.into_iter().next().unwrap())
        } else if values.is_empty() {
            Err(Error::parse_error(
                "Expected declaration value",
                position.line,
                position.column,
            ))
        } else {
            Ok(Expression::list(values, ListSeparator::Space, position))
        }
    }

    /// Parse an expression
    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_or_expression()
    }

    /// Parse a function argument expression
    /// This method is similar to parse_expression but doesn't greedily parse comma-separated lists
    /// since commas are used as argument separators in function calls
    fn parse_function_argument(&mut self) -> Result<Expression> {
        self.parse_or_expression_no_comma_list()
    }

    /// Parse OR expression without comma list expansion
    fn parse_or_expression_no_comma_list(&mut self) -> Result<Expression> {
        let mut expr = self.parse_and_expression_no_comma_list()?;

        while self.match_token(TokenType::Or) {
            let operator = BinaryOperator::Or;
            let right = self.parse_and_expression_no_comma_list()?;
            let position = expr.position().clone();
            expr = Expression::binary_op(expr, operator, right, position);
        }

        Ok(expr)
    }

    /// Parse AND expression without comma list expansion
    fn parse_and_expression_no_comma_list(&mut self) -> Result<Expression> {
        let mut expr = self.parse_equality_expression_no_comma_list()?;

        while self.match_token(TokenType::And) {
            let operator = BinaryOperator::And;
            let right = self.parse_equality_expression_no_comma_list()?;
            let position = expr.position().clone();
            expr = Expression::binary_op(expr, operator, right, position);
        }

        Ok(expr)
    }

    /// Parse equality expression without comma list expansion
    fn parse_equality_expression_no_comma_list(&mut self) -> Result<Expression> {
        let mut expr = self.parse_comparison_expression_no_comma_list()?;

        while matches!(
            self.current_token().token_type,
            TokenType::Equal | TokenType::NotEqual
        ) {
            let operator = match self.current_token().token_type {
                TokenType::Equal => {
                    self.advance();
                    BinaryOperator::Equal
                }
                TokenType::NotEqual => {
                    self.advance();
                    BinaryOperator::NotEqual
                }
                _ => unreachable!(),
            };

            let right = self.parse_comparison_expression_no_comma_list()?;
            let position = expr.position().clone();
            expr = Expression::binary_op(expr, operator, right, position);
        }

        Ok(expr)
    }

    /// Parse comparison expression without comma list expansion
    fn parse_comparison_expression_no_comma_list(&mut self) -> Result<Expression> {
        let mut expr = self.parse_additive_expression_no_comma_list()?;

        while matches!(
            self.current_token().token_type,
            TokenType::GreaterThan
                | TokenType::LessThan
                | TokenType::GreaterThanOrEqual
                | TokenType::LessThanOrEqual
        ) {
            let operator = match self.current_token().token_type {
                TokenType::GreaterThan => {
                    self.advance();
                    BinaryOperator::GreaterThan
                }
                TokenType::LessThan => {
                    self.advance();
                    BinaryOperator::LessThan
                }
                TokenType::GreaterThanOrEqual => {
                    self.advance();
                    BinaryOperator::GreaterThanOrEqual
                }
                TokenType::LessThanOrEqual => {
                    self.advance();
                    BinaryOperator::LessThanOrEqual
                }
                _ => unreachable!(),
            };

            let right = self.parse_additive_expression_no_comma_list()?;
            let position = expr.position().clone();
            expr = Expression::binary_op(expr, operator, right, position);
        }

        Ok(expr)
    }

    /// Parse additive expression without comma list expansion
    fn parse_additive_expression_no_comma_list(&mut self) -> Result<Expression> {
        let mut expr = self.parse_multiplicative_expression_no_comma_list()?;

        while matches!(
            self.current_token().token_type,
            TokenType::Plus | TokenType::Minus
        ) {
            let operator = match self.current_token().token_type {
                TokenType::Plus => {
                    self.advance();
                    BinaryOperator::Add
                }
                TokenType::Minus => {
                    self.advance();
                    BinaryOperator::Subtract
                }
                _ => unreachable!(),
            };

            let right = self.parse_multiplicative_expression_no_comma_list()?;
            let position = expr.position().clone();
            expr = Expression::binary_op(expr, operator, right, position);
        }

        Ok(expr)
    }

    /// Parse multiplicative expression without comma list expansion
    fn parse_multiplicative_expression_no_comma_list(&mut self) -> Result<Expression> {
        let mut expr = self.parse_unary_expression_no_comma_list()?;

        while matches!(
            self.current_token().token_type,
            TokenType::Multiply | TokenType::Divide
        ) {
            let operator = match self.current_token().token_type {
                TokenType::Multiply => {
                    self.advance();
                    BinaryOperator::Multiply
                }
                TokenType::Divide => {
                    self.advance();
                    BinaryOperator::Divide
                }
                _ => unreachable!(),
            };

            let right = self.parse_unary_expression_no_comma_list()?;
            let position = expr.position().clone();
            expr = Expression::binary_op(expr, operator, right, position);
        }

        Ok(expr)
    }

    /// Parse unary expression without comma list expansion
    fn parse_unary_expression_no_comma_list(&mut self) -> Result<Expression> {
        let position = self.current_position();

        if matches!(
            self.current_token().token_type,
            TokenType::Minus | TokenType::Plus | TokenType::Not
        ) {
            let operator = match self.current_token().token_type {
                TokenType::Minus => {
                    self.advance();
                    UnaryOperator::Minus
                }
                TokenType::Plus => {
                    self.advance();
                    UnaryOperator::Plus
                }
                TokenType::Not => {
                    self.advance();
                    UnaryOperator::Not
                }
                _ => unreachable!(),
            };

            let operand = self.parse_unary_expression_no_comma_list()?;
            Ok(Expression::unary_op(operator, operand, position))
        } else {
            self.parse_primary_expression_no_comma_list()
        }
    }

    /// Parse primary expression without comma list expansion for variables
    /// This is used in function argument parsing to avoid treating commas as list separators
    fn parse_primary_expression_no_comma_list(&mut self) -> Result<Expression> {
        let position = self.current_position();

        match &self.current_token().token_type {
            TokenType::String(s) => {
                let value = s.clone();
                self.advance();

                if value.contains("@{") {
                    self.parse_template_string(&value, position)
                } else {
                    Ok(Expression::string(value, position))
                }
            }
            TokenType::Number(n) => {
                let value = *n;
                self.advance();
                if let TokenType::Identifier(unit) = &self.current_token().token_type {
                    if is_css_unit(unit) {
                        let unit_str = unit.clone();
                        self.advance();
                        Ok(Expression::number_with_unit(value, unit_str, position))
                    } else {
                        Ok(Expression::number(value, position))
                    }
                } else {
                    Ok(Expression::number(value, position))
                }
            }
            TokenType::Percentage(n) => {
                let value = *n;
                self.advance();
                Ok(Expression::Percentage(value, position))
            }
            TokenType::AtKeyword(name) => {
                let var_name = name.clone();
                self.advance();
                // In function argument context, just return the variable without comma list expansion
                Ok(Expression::variable(var_name, position))
            }
            TokenType::VariableInterpolation(name) => {
                let var_name = name.clone();
                self.advance();
                Ok(Expression::Interpolation(var_name, position))
            }
            TokenType::Hash(color) => {
                let hex = format!("#{}", color);
                self.advance();
                match Expression::color_hex(&hex, position.clone()) {
                    Ok(color_expr) => Ok(color_expr),
                    Err(_) => Ok(Expression::string(hex, position)),
                }
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();

                // Check if this is a function call
                if self.match_token(TokenType::LeftParen) {
                    let mut arguments = Vec::new();

                    if !self.check(&TokenType::RightParen) {
                        loop {
                            arguments.push(self.parse_function_argument()?);
                            if !self.match_token(TokenType::Comma) {
                                break;
                            }
                        }
                    }

                    self.consume(
                        TokenType::RightParen,
                        "Expected ')' after function arguments",
                    )?;

                    Ok(Expression::function_call(name, arguments, position))
                } else {
                    Ok(Expression::identifier(name, position))
                }
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.parse_function_argument()?;
                self.consume(TokenType::RightParen, "Expected ')' after expression")?;
                Ok(Expression::parenthesized(expr, position))
            }
            _ => Err(Error::parse_error(
                format!(
                    "Unexpected token in function argument: {:?}",
                    self.current_token().token_type
                ),
                position.line,
                position.column,
            )),
        }
    }

    /// Parse a template string with variable interpolation
    fn parse_template_string(&self, text: &str, position: Position) -> Result<Expression> {
        let mut parts = Vec::new();
        let mut current_text = String::new();
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '@' && chars.peek() == Some(&'{') {
                // Found start of interpolation
                if !current_text.is_empty() {
                    parts.push(TemplateStringPart::Text(current_text.clone()));
                    current_text.clear();
                }

                chars.next(); // consume '{'
                let mut var_name = String::new();
                let mut brace_count = 1;

                while let Some(ch) = chars.next() {
                    if ch == '{' {
                        brace_count += 1;
                        var_name.push(ch);
                    } else if ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            break;
                        } else {
                            var_name.push(ch);
                        }
                    } else {
                        var_name.push(ch);
                    }
                }

                if brace_count != 0 {
                    return Err(Error::parse_error(
                        "Unterminated variable interpolation in string",
                        position.line,
                        position.column,
                    ));
                }

                parts.push(TemplateStringPart::Interpolation(var_name));
            } else {
                current_text.push(ch);
            }
        }

        if !current_text.is_empty() {
            parts.push(TemplateStringPart::Text(current_text));
        }

        Ok(Expression::TemplateString { parts, position })
    }

    /// Parse OR expression
    fn parse_or_expression(&mut self) -> Result<Expression> {
        let mut expr = self.parse_and_expression()?;

        while self.match_token(TokenType::Or) {
            let operator = BinaryOperator::Or;
            let right = self.parse_and_expression()?;
            let position = expr.position().clone();
            expr = Expression::binary_op(expr, operator, right, position);
        }

        Ok(expr)
    }

    /// Parse AND expression
    fn parse_and_expression(&mut self) -> Result<Expression> {
        let mut expr = self.parse_equality_expression()?;

        while self.match_token(TokenType::And) {
            let operator = BinaryOperator::And;
            let right = self.parse_equality_expression()?;
            let position = expr.position().clone();
            expr = Expression::binary_op(expr, operator, right, position);
        }

        Ok(expr)
    }

    /// Parse equality expression
    fn parse_equality_expression(&mut self) -> Result<Expression> {
        let mut expr = self.parse_comparison_expression()?;

        while matches!(
            self.current_token().token_type,
            TokenType::Equal | TokenType::NotEqual
        ) {
            let operator = match self.current_token().token_type {
                TokenType::Equal => {
                    self.advance();
                    BinaryOperator::Equal
                }
                TokenType::NotEqual => {
                    self.advance();
                    BinaryOperator::NotEqual
                }
                _ => unreachable!(),
            };

            let right = self.parse_comparison_expression()?;
            let position = expr.position().clone();
            expr = Expression::binary_op(expr, operator, right, position);
        }

        Ok(expr)
    }

    /// Parse comparison expression
    fn parse_comparison_expression(&mut self) -> Result<Expression> {
        let mut expr = self.parse_term_expression()?;

        while matches!(
            self.current_token().token_type,
            TokenType::LessThan
                | TokenType::LessThanOrEqual
                | TokenType::GreaterThan
                | TokenType::GreaterThanOrEqual
        ) {
            let operator = match self.current_token().token_type {
                TokenType::LessThan => {
                    self.advance();
                    BinaryOperator::LessThan
                }
                TokenType::LessThanOrEqual => {
                    self.advance();
                    BinaryOperator::LessThanOrEqual
                }
                TokenType::GreaterThan => {
                    self.advance();
                    BinaryOperator::GreaterThan
                }
                TokenType::GreaterThanOrEqual => {
                    self.advance();
                    BinaryOperator::GreaterThanOrEqual
                }
                _ => unreachable!(),
            };

            let right = self.parse_term_expression()?;
            let position = expr.position().clone();
            expr = Expression::binary_op(expr, operator, right, position);
        }

        Ok(expr)
    }

    /// Parse term expression (addition and subtraction)
    fn parse_term_expression(&mut self) -> Result<Expression> {
        let mut expr = self.parse_factor_expression()?;

        while matches!(
            self.current_token().token_type,
            TokenType::Plus | TokenType::Minus
        ) {
            let operator = match self.current_token().token_type {
                TokenType::Plus => {
                    self.advance();
                    BinaryOperator::Add
                }
                TokenType::Minus => {
                    self.advance();
                    BinaryOperator::Subtract
                }
                _ => unreachable!(),
            };

            let right = self.parse_factor_expression()?;
            let position = expr.position().clone();
            expr = Expression::binary_op(expr, operator, right, position);
        }

        Ok(expr)
    }

    /// Parse factor expression (multiplication, division, modulo)
    fn parse_factor_expression(&mut self) -> Result<Expression> {
        let mut expr = self.parse_unary_expression()?;

        while matches!(
            self.current_token().token_type,
            TokenType::Multiply | TokenType::Divide | TokenType::Modulo
        ) {
            let operator = match self.current_token().token_type {
                TokenType::Multiply => {
                    self.advance();
                    BinaryOperator::Multiply
                }
                TokenType::Divide => {
                    self.advance();
                    BinaryOperator::Divide
                }
                TokenType::Modulo => {
                    self.advance();
                    BinaryOperator::Modulo
                }
                _ => unreachable!(),
            };

            let right = self.parse_unary_expression()?;
            let position = expr.position().clone();
            expr = Expression::binary_op(expr, operator, right, position);
        }

        Ok(expr)
    }

    /// Parse unary expression
    fn parse_unary_expression(&mut self) -> Result<Expression> {
        if matches!(
            self.current_token().token_type,
            TokenType::Minus | TokenType::Plus | TokenType::Not
        ) {
            let position = self.current_position();
            let operator = match self.current_token().token_type {
                TokenType::Minus => {
                    self.advance();
                    UnaryOperator::Minus
                }
                TokenType::Plus => {
                    self.advance();
                    UnaryOperator::Plus
                }
                TokenType::Not => {
                    self.advance();
                    UnaryOperator::Not
                }
                _ => unreachable!(),
            };

            let operand = self.parse_unary_expression()?;
            Ok(Expression::unary_op(operator, operand, position))
        } else {
            self.parse_primary_expression()
        }
    }

    /// Parse primary expression
    fn parse_primary_expression(&mut self) -> Result<Expression> {
        let position = self.current_position();

        match &self.current_token().token_type {
            TokenType::String(s) => {
                let value = s.clone();
                self.advance();

                // Check if the string contains variable interpolation
                let first_expr = if value.contains("@{") {
                    self.parse_template_string(&value, position.clone())?
                } else {
                    Expression::string(value, position.clone())
                };

                // Check for comma-separated list (e.g., font-family: "Times New Roman", serif)
                let mut values = vec![first_expr];

                // Skip whitespace
                while matches!(self.current_token().token_type, TokenType::Whitespace) {
                    self.advance();
                }

                // Check for comma-separated values
                while self.check(&TokenType::Comma) {
                    // Look ahead to see what comes after the comma
                    let mut lookahead = self.current + 1;

                    // Skip whitespace in lookahead
                    while lookahead < self.tokens.len()
                        && matches!(self.tokens[lookahead].token_type, TokenType::Whitespace)
                    {
                        lookahead += 1;
                    }

                    // Check if the token after comma is a valid list item
                    let is_valid_list_item = if lookahead < self.tokens.len() {
                        matches!(
                            self.tokens[lookahead].token_type,
                            TokenType::Identifier(_)
                                | TokenType::AtKeyword(_)
                                | TokenType::String(_)
                        )
                    } else {
                        false
                    };

                    if !is_valid_list_item {
                        break;
                    }

                    // Consume the comma
                    self.advance();

                    // Skip whitespace after comma
                    while matches!(self.current_token().token_type, TokenType::Whitespace) {
                        self.advance();
                    }

                    // Parse next value
                    match &self.current_token().token_type {
                        TokenType::Identifier(id) => {
                            values
                                .push(Expression::identifier(id.clone(), self.current_position()));
                            self.advance();
                        }
                        TokenType::AtKeyword(var) => {
                            values.push(Expression::variable(var.clone(), self.current_position()));
                            self.advance();
                        }
                        TokenType::String(s) => {
                            let val = s.clone();
                            let pos = self.current_position();
                            self.advance();
                            if val.contains("@{") {
                                values.push(self.parse_template_string(&val, pos)?);
                            } else {
                                values.push(Expression::string(val, pos));
                            }
                        }
                        _ => break,
                    }
                }

                if values.len() == 1 {
                    Ok(values.into_iter().next().unwrap())
                } else {
                    Ok(Expression::list(values, ListSeparator::Comma, position))
                }
            }
            TokenType::Number(n) => {
                let value = *n;
                self.advance();
                // Check if there's a unit following
                if let TokenType::Identifier(unit) = &self.current_token().token_type {
                    // Only treat as unit if it's a valid CSS unit, not a keyword
                    if is_css_unit(unit) {
                        let unit_str = unit.clone();
                        self.advance();
                        Ok(Expression::number_with_unit(value, unit_str, position))
                    } else {
                        // It's a CSS keyword (like 'auto'), not a unit
                        Ok(Expression::number(value, position))
                    }
                } else {
                    Ok(Expression::number(value, position))
                }
            }
            TokenType::Percentage(n) => {
                let value = *n;
                self.advance();
                Ok(Expression::Percentage(value, position))
            }
            TokenType::AtKeyword(name) => {
                let var_name = name.clone();
                self.advance();

                // Parse as comma-separated list if more tokens follow
                let mut values = vec![Expression::variable(var_name, position.clone())];

                // Skip whitespace
                while matches!(self.current_token().token_type, TokenType::Whitespace) {
                    self.advance();
                }

                // Check for comma-separated values, but be conservative about it
                // Only parse as comma list if the next token after comma looks like a valid list item
                while self.check(&TokenType::Comma) {
                    // Look ahead to see what comes after the comma
                    let mut lookahead = self.current + 1;

                    // Skip whitespace in lookahead
                    while lookahead < self.tokens.len()
                        && matches!(self.tokens[lookahead].token_type, TokenType::Whitespace)
                    {
                        lookahead += 1;
                    }

                    // Check if the token after comma is a valid list item
                    let is_valid_list_item = if lookahead < self.tokens.len() {
                        matches!(
                            self.tokens[lookahead].token_type,
                            TokenType::Identifier(_)
                                | TokenType::AtKeyword(_)
                                | TokenType::String(_)
                        )
                    } else {
                        false
                    };

                    if !is_valid_list_item {
                        break;
                    }

                    // Consume the comma
                    self.advance();

                    // Skip whitespace after comma
                    while matches!(self.current_token().token_type, TokenType::Whitespace) {
                        self.advance();
                    }

                    // Parse next value
                    match &self.current_token().token_type {
                        TokenType::Identifier(id) => {
                            values
                                .push(Expression::identifier(id.clone(), self.current_position()));
                            self.advance();
                        }
                        TokenType::AtKeyword(var) => {
                            values.push(Expression::variable(var.clone(), self.current_position()));
                            self.advance();
                        }
                        TokenType::String(s) => {
                            let val = s.clone();
                            let pos = self.current_position();
                            self.advance();
                            if val.contains("@{") {
                                values.push(self.parse_template_string(&val, pos)?);
                            } else {
                                values.push(Expression::string(val, pos));
                            }
                        }
                        _ => break,
                    }
                }

                if values.len() == 1 {
                    Ok(values.into_iter().next().unwrap())
                } else {
                    Ok(Expression::list(values, ListSeparator::Comma, position))
                }
            }
            TokenType::VariableInterpolation(name) => {
                let var_name = name.clone();
                self.advance();
                Ok(Expression::Interpolation(var_name, position))
            }
            TokenType::Hash(color) => {
                let hex = format!("#{}", color);
                self.advance();
                match Expression::color_hex(&hex, position.clone()) {
                    Ok(color_expr) => Ok(color_expr),
                    Err(_) => Ok(Expression::string(hex, position)),
                }
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();

                // Check if this is a function call
                if self.match_token(TokenType::LeftParen) {
                    let mut arguments = Vec::new();

                    if !self.check(&TokenType::RightParen) {
                        loop {
                            // Use parse_function_argument to avoid comma list expansion
                            arguments.push(self.parse_function_argument()?);
                            if !self.match_token(TokenType::Comma) {
                                break;
                            }
                        }
                    }

                    self.consume(
                        TokenType::RightParen,
                        "Expected ')' after function arguments",
                    )?;

                    Ok(Expression::function_call(name, arguments, position))
                } else {
                    // Parse as comma-separated list if more tokens follow
                    let mut values = vec![Expression::identifier(name, position.clone())];

                    // Skip whitespace
                    while matches!(self.current_token().token_type, TokenType::Whitespace) {
                        self.advance();
                    }

                    // Check for comma-separated values
                    while self.match_token(TokenType::Comma) {
                        // Skip whitespace after comma
                        while matches!(self.current_token().token_type, TokenType::Whitespace) {
                            self.advance();
                        }

                        // Parse next value
                        match &self.current_token().token_type {
                            TokenType::Identifier(id) => {
                                values
                                    .push(Expression::string(id.clone(), self.current_position()));
                                self.advance();
                            }
                            TokenType::AtKeyword(var) => {
                                values.push(Expression::variable(
                                    var.clone(),
                                    self.current_position(),
                                ));
                                self.advance();
                            }
                            TokenType::String(s) => {
                                let val = s.clone();
                                let pos = self.current_position();
                                self.advance();
                                if val.contains("@{") {
                                    values.push(self.parse_template_string(&val, pos)?);
                                } else {
                                    values.push(Expression::string(val, pos));
                                }
                            }
                            _ => break,
                        }
                    }

                    if values.len() == 1 {
                        Ok(values.into_iter().next().unwrap())
                    } else {
                        Ok(Expression::list(values, ListSeparator::Comma, position))
                    }
                }
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(TokenType::RightParen, "Expected ')' after expression")?;
                Ok(Expression::parenthesized(expr, position))
            }
            _ => Err(Error::parse_error(
                "Expected expression",
                position.line,
                position.column,
            )),
        }
    }
}

/// Check if an identifier is a valid CSS unit
fn is_css_unit(unit: &str) -> bool {
    matches!(
        unit,
        // Length units
        "px" | "em" | "rem" | "ex" | "ch" | "vw" | "vh" | "vmin" | "vmax" |
        "cm" | "mm" | "in" | "pt" | "pc" | "q" |
        // Angle units
        "deg" | "grad" | "rad" | "turn" |
        // Time units
        "s" | "ms" |
        // Frequency units
        "hz" | "khz" |
        // Resolution units
        "dpi" | "dpcm" | "dppx" |
        // Percentage (although % is handled separately)
        "%" |
        // Other common units
        "fr"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_variable_declaration() {
        let mut parser = Parser::from_string("@color: red;".to_string()).unwrap();
        let stylesheet = parser.parse().unwrap();

        assert_eq!(stylesheet.statements.len(), 1);
        if let Statement::Variable(var) = &stylesheet.statements[0] {
            assert_eq!(var.name, "color");
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_simple_rule() {
        let mut parser = Parser::from_string(".test { color: red; }".to_string()).unwrap();
        let stylesheet = parser.parse().unwrap();

        assert_eq!(stylesheet.statements.len(), 1);
        if let Statement::Rule(rule) = &stylesheet.statements[0] {
            assert_eq!(rule.selectors.len(), 1);
            assert_eq!(rule.declarations.len(), 1);
        } else {
            panic!("Expected rule");
        }
    }

    #[test]
    fn test_parse_number_with_unit() {
        let mut parser = Parser::from_string(".test { width: 10px; }".to_string()).unwrap();
        let stylesheet = parser.parse().unwrap();

        if let Statement::Rule(rule) = &stylesheet.statements[0] {
            if let Some(decl) = rule.declarations.first() {
                assert_eq!(decl.property, "width");
                // Check that value is parsed correctly
                match &decl.value {
                    Expression::Number {
                        value,
                        unit: Some(unit),
                        ..
                    } => {
                        assert_eq!(*value, 10.0);
                        assert_eq!(unit, "px");
                    }
                    _ => panic!("Expected number with unit"),
                }
            }
        }
    }

    #[test]
    fn test_mixin_detection() {
        // Test 1: Simple parameterized mixin should be detected as definition
        let mut parser1 =
            Parser::from_string(".border-radius(@radius) { border-radius: @radius; }".to_string())
                .unwrap();
        parser1.current = 0;
        assert!(
            parser1.is_mixin_definition(),
            "Should detect .border-radius(@radius) {{ as mixin definition"
        );
        assert!(
            !parser1.is_mixin_call(),
            "Should not detect .border-radius(@radius) {{ as mixin call"
        );

        // Test 2: Mixin call should be detected as call
        let mut parser2 = Parser::from_string(".border-radius(5px);".to_string()).unwrap();
        parser2.current = 0;
        assert!(
            !parser2.is_mixin_definition(),
            "Should not detect .border-radius(5px); as mixin definition"
        );
        assert!(
            parser2.is_mixin_call(),
            "Should detect .border-radius(5px); as mixin call"
        );

        // Test 3: CSS rule should not be detected as either
        let mut parser3 = Parser::from_string(".button { color: red; }".to_string()).unwrap();
        parser3.current = 0;
        assert!(
            !parser3.is_mixin_definition(),
            "Should not detect .button {{ as mixin definition"
        );
        assert!(
            !parser3.is_mixin_call(),
            "Should not detect .button {{ as mixin call"
        );
    }

    #[test]
    fn test_mixin_parsing() {
        // Test parsing a simple mixin definition
        let mut parser =
            Parser::from_string(".border-radius(@radius) { border-radius: @radius; }".to_string())
                .unwrap();
        let result = parser.parse();

        match result {
            Ok(stylesheet) => {
                assert_eq!(stylesheet.statements.len(), 1);
                if let Statement::MixinDefinition(mixin) = &stylesheet.statements[0] {
                    assert_eq!(mixin.name, ".border-radius");
                    assert_eq!(mixin.parameters.len(), 1);
                    assert_eq!(mixin.parameters[0].name, "radius");
                } else {
                    panic!(
                        "Expected mixin definition, got: {:?}",
                        stylesheet.statements[0]
                    );
                }
            }
            Err(e) => {
                panic!("Failed to parse mixin definition: {}", e);
            }
        }
    }

    #[test]
    fn test_mixin_full_usage() {
        // Test parsing the full mixin usage scenario
        let input = r#".border-radius(@radius) {
    border-radius: @radius;
    -webkit-border-radius: @radius;
    -moz-border-radius: @radius;
}

.button {
    .border-radius(5px);
    padding: 10px;
}"#;

        let mut parser = Parser::from_string(input.to_string()).unwrap();
        let result = parser.parse();

        match result {
            Ok(stylesheet) => {
                // Should have 2 statements: mixin definition and rule
                assert_eq!(stylesheet.statements.len(), 2);

                // First should be mixin definition
                if let Statement::MixinDefinition(mixin) = &stylesheet.statements[0] {
                    assert_eq!(mixin.name, ".border-radius");
                    assert_eq!(mixin.parameters.len(), 1);
                    assert_eq!(mixin.parameters[0].name, "radius");
                } else {
                    panic!(
                        "Expected mixin definition, got: {:?}",
                        stylesheet.statements[0]
                    );
                }

                // Second should be rule with mixin call and declaration
                if let Statement::Rule(rule) = &stylesheet.statements[1] {
                    assert_eq!(rule.declarations.len(), 1);
                    assert_eq!(rule.nested_rules.len(), 1);

                    // Verify we have the correct content
                    assert_eq!(rule.declarations[0].property, "padding");
                    if let Statement::MixinCall(call) = &rule.nested_rules[0] {
                        assert_eq!(call.name, ".border-radius");
                    } else {
                        panic!("Expected mixin call in nested rules");
                    }
                } else {
                    panic!("Expected rule, got: {:?}", stylesheet.statements[1]);
                }
            }
            Err(e) => {
                panic!("Failed to parse mixin usage: {}", e);
            }
        }
    }

    #[test]
    fn test_multi_variable_expression_parsing() {
        // Test parsing a declaration with multiple variables
        let input = r#".test {
    box-shadow: @x @y @blur @color;
}"#;

        let mut parser = Parser::from_string(input.to_string()).unwrap();
        let result = parser.parse();

        match result {
            Ok(stylesheet) => {
                assert_eq!(stylesheet.statements.len(), 1);

                if let Statement::Rule(rule) = &stylesheet.statements[0] {
                    assert_eq!(rule.declarations.len(), 1);
                    let decl = &rule.declarations[0];
                    assert_eq!(decl.property, "box-shadow");

                    // Debug the parsed expression structure
                    println!("Declaration value: {:?}", decl.value);

                    // The value should be a list of variables
                    match &decl.value {
                        Expression::List { values, .. } => {
                            println!("Found list with {} values", values.len());
                            for (i, val) in values.iter().enumerate() {
                                println!("  Value {}: {:?}", i, val);
                            }
                        }
                        other => {
                            println!("Not a list, but: {:?}", other);
                        }
                    }
                } else {
                    panic!("Expected rule");
                }
            }
            Err(e) => {
                panic!("Failed to parse multi-variable expression: {}", e);
            }
        }
    }

    #[test]
    fn test_mixin_default_parameters() {
        // Test parsing and default parameter handling
        let input = r#".test-mixin(@param: default-value) {
    property: @param;
}

.usage {
    .test-mixin();
}"#;

        let mut parser = Parser::from_string(input.to_string()).unwrap();
        let result = parser.parse();

        match result {
            Ok(stylesheet) => {
                assert_eq!(stylesheet.statements.len(), 2);

                // First should be mixin definition with default parameter
                if let Statement::MixinDefinition(mixin) = &stylesheet.statements[0] {
                    assert_eq!(mixin.name, ".test-mixin");
                    assert_eq!(mixin.parameters.len(), 1);
                    assert_eq!(mixin.parameters[0].name, "param");
                    assert!(mixin.parameters[0].default_value.is_some());
                } else {
                    panic!("Expected mixin definition");
                }

                // Second should be rule with mixin call
                if let Statement::Rule(rule) = &stylesheet.statements[1] {
                    assert_eq!(rule.nested_rules.len(), 1);
                    if let Statement::MixinCall(call) = &rule.nested_rules[0] {
                        assert_eq!(call.name, ".test-mixin");
                        assert_eq!(call.arguments.len(), 0); // No arguments provided
                    } else {
                        panic!("Expected mixin call");
                    }
                } else {
                    panic!("Expected rule");
                }
            }
            Err(e) => {
                panic!("Failed to parse mixin with default parameters: {}", e);
            }
        }
    }
}
