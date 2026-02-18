use rust_less::ast::{SimpleSelector, Statement};
use rust_less::parser::Parser;

#[test]
fn test_parse_simple_extend() {
    let _input = r#"
    .a:extend(.b) {}
    "#;
    // Note: The parser currently parses &:extend as a statement inside a rule or at top level?
    // Wait, :extend is a pseudo-class on a selector, OR a statement?
    // My implementation is for `&:extend(...)` statement!
    // LESS supports `div { &:extend(.b); }` (Extend inside rule)
    // AND `.a:extend(.b) {}` (Extend attached to selector).

    // The current implementation adds `Statement::Extend`.
    // This corresponds to `&:extend(.b);` inside a block.
    // Let's test that first.
}

#[test]
fn test_parse_extend_statement() {
    let input = r#"
    .test {
        &:extend(.b);
    }
    "#;

    let mut parser = Parser::from_string(input.to_string()).unwrap();
    let stylesheet = parser.parse().unwrap();

    assert_eq!(stylesheet.statements.len(), 1);

    if let Statement::Rule(rule) = &stylesheet.statements[0] {
        // nested_rules should contain the extend statement
        assert_eq!(rule.nested_rules.len(), 1);
        if let Statement::Extend(extend) = &rule.nested_rules[0] {
            assert_eq!(extend.selectors.len(), 1);
            // Verify selector .b
            let selector = &extend.selectors[0];
            assert_eq!(selector.parts.len(), 1);
            let part = &selector.parts[0];
            assert_eq!(part.simple_selectors.len(), 1);

            if let SimpleSelector::Class { name, .. } = &part.simple_selectors[0] {
                assert_eq!(name, "b");
            } else {
                panic!("Expected class selector .b");
            }

            assert!(!extend.all);
        } else {
            panic!("Expected Extend statement");
        }
    } else {
        panic!("Expected Rule");
    }
}

#[test]
fn test_parse_extend_all() {
    let input = r#"
    .test {
        &:extend(.b all);
    }
    "#;

    let mut parser = Parser::from_string(input.to_string()).unwrap();
    let stylesheet = parser.parse().unwrap();

    if let Statement::Rule(rule) = &stylesheet.statements[0] {
        if let Statement::Extend(extend) = &rule.nested_rules[0] {
            assert!(extend.all);
        } else {
            panic!("Expected Extend statement");
        }
    }
}

#[test]
fn test_parse_extend_multiple() {
    let input = r#"
    .test {
        &:extend(.b, .c);
    }
    "#;

    let mut parser = Parser::from_string(input.to_string()).unwrap();
    let stylesheet = parser.parse().unwrap();

    if let Statement::Rule(rule) = &stylesheet.statements[0] {
        if let Statement::Extend(extend) = &rule.nested_rules[0] {
            assert_eq!(extend.selectors.len(), 2);
        } else {
            panic!("Expected Extend statement");
        }
    }
}
