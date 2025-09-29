use rust_less::compile;

fn main() {
    println!("Testing undefined variable behavior...\n");

    // Test the exact failing case
    let less = r#"
.test {
    color: @undefined-variable;
}
"#;

    println!("LESS input:");
    println!("{}", less);
    println!("\n{}", "=".repeat(50));

    match compile(less) {
        Ok(css) => {
            println!("Unexpected SUCCESS! Compiled CSS:");
            println!("{}", css);
        }
        Err(e) => {
            println!("ERROR (as expected): {:?}", e);

            // Check if it's the right type of error
            match e {
                rust_less::Error::UndefinedVariable { .. } => {
                    println!("✅ Correct error type: UndefinedVariable");
                }
                rust_less::Error::ParseError { .. } => {
                    println!("❌ Wrong error type: ParseError (should be UndefinedVariable)");
                }
                rust_less::Error::CompilationError { .. } => {
                    println!("❌ Wrong error type: CompilerError (should be UndefinedVariable)");
                }
                _ => {
                    println!("❌ Other error type: {:?}", e);
                }
            }
        }
    }

    // Test other undefined variable scenarios
    println!("\n{}", "=".repeat(50));
    println!("Testing other undefined variable scenarios:");

    let test_cases = vec![
        (
            "Undefined in function",
            r#".test { width: lighten(@undefined, 10%); }"#,
        ),
        (
            "Undefined in calculation",
            r#".test { width: @undefined + 10px; }"#,
        ),
        (
            "Undefined interpolation",
            r#".test-@{undefined} { color: red; }"#,
        ),
        ("Multiple undefined", r#".test { margin: @a @b @c @d; }"#),
    ];

    for (name, less) in test_cases {
        println!("\n{}: {}", name, less);
        match compile(less) {
            Ok(css) => println!("  Unexpected Success: {}", css.trim()),
            Err(e) => {
                println!("  Error: {:?}", e);
                match e {
                    rust_less::Error::UndefinedVariable { .. } => {
                        println!("    ✅ Correct error type")
                    }
                    _ => println!("    ❌ Wrong error type"),
                }
            }
        }
    }

    // Test with some defined variables for comparison
    println!("\n{}", "=".repeat(50));
    println!("Testing with defined variables (should work):");

    let working_cases = vec![
        (
            "Simple variable",
            r#"@color: red; .test { color: @color; }"#,
        ),
        (
            "Variable in function",
            r#"@color: #333; .test { color: lighten(@color, 10%); }"#,
        ),
        (
            "Variable calculation",
            r#"@width: 10px; .test { width: @width + 5px; }"#,
        ),
    ];

    for (name, less) in working_cases {
        println!("\n{}: {}", name, less);
        match compile(less) {
            Ok(css) => println!("  Success: {}", css.trim()),
            Err(e) => println!("  Unexpected Error: {:?}", e),
        }
    }
}
