use rust_less::compile;

fn main() {
    println!("Testing string functions...\n");

    // Test the exact failing case
    let less = r#"
@str: "hello world";

.strings {
    escaped: e(@str);
    replaced: replace(@str, "world", "LESS");
}
"#;

    println!("LESS input:");
    println!("{}", less);
    println!("\n{}", "=".repeat(50));

    match compile(less) {
        Ok(css) => {
            println!("SUCCESS! Compiled CSS:");
            println!("{}", css);
        }
        Err(e) => {
            println!("ERROR: {:?}", e);

            // Test individual components
            println!("\n{}", "=".repeat(50));
            println!("Testing individual string functions:");

            // Test 1: Just the variable
            let var_only = r#"
@str: "hello world";
.test { content: @str; }
"#;
            println!("\nTest 1 - Variable only:");
            match compile(var_only) {
                Ok(css) => println!("SUCCESS: {}", css.trim()),
                Err(e) => println!("ERROR: {:?}", e),
            }

            // Test 2: Direct escape function
            let direct_escape = r#"
.test { content: e("hello world"); }
"#;
            println!("\nTest 2 - Direct escape function:");
            match compile(direct_escape) {
                Ok(css) => println!("SUCCESS: {}", css.trim()),
                Err(e) => println!("ERROR: {:?}", e),
            }

            // Test 3: Escape function with variable
            let escape_var = r#"
@str: "hello world";
.test { content: e(@str); }
"#;
            println!("\nTest 3 - Escape function with variable:");
            match compile(escape_var) {
                Ok(css) => println!("SUCCESS: {}", css.trim()),
                Err(e) => println!("ERROR: {:?}", e),
            }

            // Test 4: Direct replace function
            let direct_replace = r#"
.test { content: replace("hello world", "world", "LESS"); }
"#;
            println!("\nTest 4 - Direct replace function:");
            match compile(direct_replace) {
                Ok(css) => println!("SUCCESS: {}", css.trim()),
                Err(e) => println!("ERROR: {:?}", e),
            }

            // Test 5: Replace function with variable
            let replace_var = r#"
@str: "hello world";
.test { content: replace(@str, "world", "LESS"); }
"#;
            println!("\nTest 5 - Replace function with variable:");
            match compile(replace_var) {
                Ok(css) => println!("SUCCESS: {}", css.trim()),
                Err(e) => println!("ERROR: {:?}", e),
            }

            // Test 6: Check if 'e' function is registered (alias for escape)
            let e_function = r#"
.test { content: escape("hello world"); }
"#;
            println!("\nTest 6 - Escape function (full name):");
            match compile(e_function) {
                Ok(css) => println!("SUCCESS: {}", css.trim()),
                Err(e) => println!("ERROR: {:?}", e),
            }
        }
    }

    // Test additional string functions
    println!("\n{}", "=".repeat(50));
    println!("Testing other string-related functionality:");

    let other_tests = vec![
        ("String literal", r#".test { content: "hello"; }"#),
        ("String with quotes", r#".test { content: 'hello'; }"#),
        (
            "String concatenation",
            r#".test { content: "hello" + " world"; }"#,
        ),
        ("URL function", r#".test { background: url("image.png"); }"#),
    ];

    for (name, less) in other_tests {
        println!("\n{}: {}", name, less);
        match compile(less) {
            Ok(css) => println!("  Result: {}", css.trim()),
            Err(e) => println!("  Error: {:?}", e),
        }
    }
}
