use rust_less::compile;

fn main() {
    println!("Testing the exact failing case...\n");

    // This is the exact test case that's failing
    let less = r#"
@base-color: #333;

.theme {
    color: @base-color;
    background: lighten(@base-color, 20%);
    border: darken(@base-color, 10%);
}
"#;

    println!("LESS input:");
    println!("{}", less);
    println!("\n{}", "=".repeat(50));

    match compile(less) {
        Ok(css) => {
            println!("SUCCESS! Compiled CSS:");
            println!("{}", css);

            let expected = r#".theme {
  color: #333;
  background: #666;
  border: #1a1a1a;
}"#;

            println!("\n{}", "=".repeat(50));
            println!("Expected:");
            println!("{}", expected);

            println!("\n{}", "=".repeat(50));
            if css.trim() == expected.trim() {
                println!("✅ TEST PASSES!");
            } else {
                println!("❌ TEST FAILS - outputs don't match");
                println!("Differences:");
                let actual_lines: Vec<&str> = css.trim().lines().collect();
                let expected_lines: Vec<&str> = expected.trim().lines().collect();

                for (i, (actual, expected)) in
                    actual_lines.iter().zip(expected_lines.iter()).enumerate()
                {
                    if actual != expected {
                        println!("Line {}: Expected '{}', got '{}'", i + 1, expected, actual);
                    }
                }
            }
        }
        Err(e) => {
            println!("ERROR: {:?}", e);

            // Try to debug what's happening step by step
            println!("\n{}", "=".repeat(50));
            println!("Debugging step by step...");

            // Test 1: Just the variable declaration
            let var_only = r#"@base-color: #333;"#;
            println!("\nTest 1 - Variable declaration only:");
            match compile(var_only) {
                Ok(css) => println!("SUCCESS: {}", css.trim()),
                Err(e) => println!("ERROR: {:?}", e),
            }

            // Test 2: Variable + simple usage
            let var_usage = r#"
@base-color: #333;
.test { color: @base-color; }
"#;
            println!("\nTest 2 - Variable usage:");
            match compile(var_usage) {
                Ok(css) => println!("SUCCESS: {}", css.trim()),
                Err(e) => println!("ERROR: {:?}", e),
            }

            // Test 3: Just one function call with variable
            let one_func = r#"
@base-color: #333;
.test { background: lighten(@base-color, 20%); }
"#;
            println!("\nTest 3 - One function call with variable:");
            match compile(one_func) {
                Ok(css) => println!("SUCCESS: {}", css.trim()),
                Err(e) => println!("ERROR: {:?}", e),
            }

            // Test 4: Check if it's the multiple properties issue
            let multi_props = r#"
@base-color: #333;
.test {
    color: @base-color;
    background: @base-color;
}
"#;
            println!("\nTest 4 - Multiple properties with variables:");
            match compile(multi_props) {
                Ok(css) => println!("SUCCESS: {}", css.trim()),
                Err(e) => println!("ERROR: {:?}", e),
            }
        }
    }
}
