use rust_less::compile;

fn main() {
    // Test 1: Simple variable test
    let simple = r#"@var: "test";
.test { color: @var; }"#;

    println!("=== Simple Variable Test ===");
    match compile(simple) {
        Ok(css) => println!("Success: {}", css),
        Err(e) => println!("Error: {}", e),
    }

    // Test 2: Comma test without variables
    let comma_test = r#"
.test {
    font-family: Arial, sans-serif;
}
"#;

    println!("\n=== Comma Test (no variables) ===");
    match compile(comma_test) {
        Ok(css) => println!("Success: {}", css),
        Err(e) => println!("Error: {}", e),
    }

    // Test 3: Variable with comma
    let var_comma = r#"
@font: "Arial";
.test {
    font-family: @font;
}
"#;

    println!("\n=== Variable Test (no comma) ===");
    match compile(var_comma) {
        Ok(css) => println!("Success: {}", css),
        Err(e) => println!("Error: {}", e),
    }

    // Test 4: Variable in comma list
    let var_comma_test = r#"
@font: "Arial";
.test {
    font-family: @font, sans-serif;
}
"#;

    println!("\n=== Variable in Comma List Test ===");
    match compile(var_comma_test) {
        Ok(css) => {
            println!("Success: {}", css);
            println!("Contains Arial: {}", css.contains("Arial"));
            println!("Contains sans-serif: {}", css.contains("sans-serif"));
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test 5: String operations test
    let less = r#"
@base-font: "Arial";
@weight: "bold";

.text {
    font-family: @base-font, sans-serif;
    font-weight: @weight;
}
"#;

    println!("\n=== String Operations Test ===");
    println!("Input LESS:");
    println!("{}", less);

    match compile(less) {
        Ok(css) => {
            println!("\nCSS Output:");
            println!("{}", css);

            let expected = r#".text {
  font-family: "Arial", sans-serif;
  font-weight: "bold";
}"#;

            println!("\nExpected:");
            println!("{}", expected);

            println!("\nMatch: {}", css.trim() == expected.trim());
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
