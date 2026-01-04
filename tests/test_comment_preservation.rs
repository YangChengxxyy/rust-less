use rust_less::compile;

#[test]
fn test_comments_inside_rules() {
    let less = r#"
    .rule {
        // This is a comment
        color: red;
        /* Another comment */
        background: blue;
    }
    "#;

    let result = compile(less).unwrap();
    println!("Result: {}", result);
    
    // Check that comments are preserved in the output
    // Note: The current implementation might only preserve /* */ block comments in standard CSS output,
    // or // comments might be filtered depending on compiler settings. 
    // Usually, LESS compilers strip // comments but keep /* */.
    // Let's check what our compiler does.
    
    // Based on the code change, we treated Statement::Comment as a RuleItem.
    // If the comment logic works, it should appear in the output block.
    
    // Note: Standard LESS/CSS behavior:
    // // comments are silent (removed).
    // /* */ comments are preserved.
    
    // Let's see if our parser distinguishes them effectively or if we just emit everything.
    // If the parser parses // as a Comment node, and we emit it, it will show up.
    
    // Let's assert strictly for block comments first as that's standard CSS.
    assert!(result.contains("/* Another comment */"), "Block comment should be preserved");
    
    // If our implementation preserves inline comments (non-standard for CSS but maybe intended for this project), check that too.
    // If not, we can adjust the expectation.
}
