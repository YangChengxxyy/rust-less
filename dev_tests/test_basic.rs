use rust_less::{compile, parser::Parser};

fn main() {
    println!("🔍 Debug Basic CSS Compilation");
    println!("===============================");

    // Test 1: Very simple CSS rule
    test_simple_rule();

    // Test 2: Variable usage
    test_variable_usage();

    // Test 3: Nesting (existing functionality)
    test_nesting();

    // Test 4: Check what happens with empty input
    test_empty_input();
}

fn test_simple_rule() {
    println!("\n🧪 Test 1: Simple CSS Rule");

    let css_code = r#"
.simple {
    color: red;
    background: blue;
}
"#;

    match compile(css_code) {
        Ok(css) => {
            println!("✅ Simple rule compilation successful");
            println!("CSS Output: '{}'", css);
            println!("Output length: {} characters", css.len());
        }
        Err(e) => {
            println!("❌ Simple rule compilation failed: {}", e);
        }
    }

    // Also test parsing
    match Parser::from_string(css_code.to_string()) {
        Ok(mut parser) => match parser.parse() {
            Ok(stylesheet) => {
                println!(
                    "✅ Parsing successful - {} statements",
                    stylesheet.statements.len()
                );
                for (i, statement) in stylesheet.statements.iter().enumerate() {
                    match statement {
                        rust_less::ast::Statement::Rule(rule) => {
                            println!(
                                "  Statement {}: Rule with {} selectors, {} declarations",
                                i,
                                rule.selectors.len(),
                                rule.declarations.len()
                            );
                        }
                        _ => {
                            println!("  Statement {}: {:?}", i, statement);
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Parsing failed: {}", e);
            }
        },
        Err(e) => {
            println!("❌ Parser creation failed: {}", e);
        }
    }
}

fn test_variable_usage() {
    println!("\n🧪 Test 2: Variable Usage (known working feature)");

    let less_code = r#"
@primary: #007cba;

.header {
    color: @primary;
    background: white;
}
"#;

    match compile(less_code) {
        Ok(css) => {
            println!("✅ Variable compilation successful");
            println!("CSS Output: '{}'", css);
            println!("Output length: {} characters", css.len());

            // Check if variable was substituted
            if css.contains("#007cba") {
                println!("✅ Variable substitution worked");
            } else {
                println!("❌ Variable substitution failed");
            }
        }
        Err(e) => {
            println!("❌ Variable compilation failed: {}", e);
        }
    }
}

fn test_nesting() {
    println!("\n🧪 Test 3: Nesting (known working feature)");

    let less_code = r#"
.nav {
    height: 60px;
    ul {
        margin: 0;
        li {
            list-style: none;
        }
    }
}
"#;

    match compile(less_code) {
        Ok(css) => {
            println!("✅ Nesting compilation successful");
            println!("CSS Output: '{}'", css);
            println!("Output length: {} characters", css.len());

            // Check if nesting was expanded
            if css.contains(".nav ul") && css.contains(".nav ul li") {
                println!("✅ Nesting expansion worked");
            } else {
                println!("❌ Nesting expansion failed");
            }
        }
        Err(e) => {
            println!("❌ Nesting compilation failed: {}", e);
        }
    }
}

fn test_empty_input() {
    println!("\n🧪 Test 4: Empty Input");

    let empty_code = "";

    match compile(empty_code) {
        Ok(css) => {
            println!("✅ Empty input compilation successful");
            println!("CSS Output: '{}'", css);
            println!("Output length: {} characters", css.len());
        }
        Err(e) => {
            println!("❌ Empty input compilation failed: {}", e);
        }
    }
}
