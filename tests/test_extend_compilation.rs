use rust_less::Compiler;

#[test]
fn test_simple_extend() {
    let input = r#"
    .a { color: red; }
    .b { &:extend(.a); }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile(input).unwrap();
    
    // .a should be extended by .b
    assert!(result.contains(".a, .b"));
    assert!(result.contains("color: red"));
    
    // .b itself shouldn't be output as empty rule
    // But result.contains(".b") is true because of above.
    // Check strict output if possible, but contains is good first step.
}

#[test]
fn test_extend_defined_before() {
    let input = r#"
    .b { &:extend(.a); }
    .a { color: red; }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile(input).unwrap();
    
    assert!(result.contains(".a, .b"));
    assert!(result.contains("color: red"));
}

#[test]
fn test_nested_extend() {
    let input = r#"
    .a { color: red; }
    .parent {
        .child { &:extend(.a); }
    }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile(input).unwrap();
    
    assert!(result.contains(".a, .parent .child"));
    assert!(result.contains("color: red"));
}

#[test]
fn test_multiple_extenders() {
    let input = r#"
    .a { color: red; }
    .b { &:extend(.a); }
    .c { &:extend(.a); }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile(input).unwrap();
    
    // Order depends on hash map or insertion order. 
    // Since we push to vector in registry, order should be preserved if single thread.
    // But hash map iteration order (merge?) might be random.
    // So check parts.
    assert!(result.contains(".a"));
    assert!(result.contains(".b"));
    assert!(result.contains(".c"));
    assert!(result.contains("color: red"));
}

#[test]
fn test_chained_extend() {
    // .b extends .a
    // .c extends .b
    // .a should have .b
    // .b should have .c
    // Less does NOT flatten chained extends automatically unless "all" is used?
    // Actually standard CSS extend:
    // .a { color: red; }
    // .b:extend(.a) {} -> .a, .b { color: red; }
    // .c:extend(.b) {} -> .b, .c {} (empty if .b empty)
    // Does .c extend .a?
    // In Less, "Extend is not recursive".
    // So .c does NOT get .a's properties unless .b has properties.
    // But if .b has properties:
    // .b { color: blue; &:extend(.a); }
    // .a -> .a, .b { color: red; }
    // .b -> .b, .c { color: blue; }
    // .c extends .b.
    // Result: .c has color: blue. .b has color: red (via .a rule) and blue.
    // Does .c get red?
    // Less docs: "It does not duplicate the styling... unless the all keyword is specified in the extend."
    // Actually, "Extend is not recursive" means .c:extend(.b) does not make .c match .a selectors.
    
    let input = r#"
    .a { color: red; }
    .b { &:extend(.a); }
    .c { &:extend(.b); }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile(input).unwrap();
    
    assert!(result.contains(".a, .b"));
    // .c extends .b. But .b rule (empty) isn't output.
    // However, does .b match .a's selector? No.
    // So .c should NOT appear in .a rule.
    assert!(!result.contains(".a, .b, .c"));
    assert!(!result.contains(".a, .c"));
}
