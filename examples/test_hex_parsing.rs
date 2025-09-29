use rust_less::compile;

fn main() {
    println!("Testing hex color parsing...\n");

    // Test basic hex color
    let simple_hex = r#"
.test {
    color: #333;
}
"#;

    println!("Testing: #333");
    match compile(simple_hex) {
        Ok(css) => println!("SUCCESS: {}", css.trim()),
        Err(e) => println!("ERROR: {:?}", e),
    }

    // Test 6-digit hex
    let hex6 = r#"
.test {
    color: #ff0000;
}
"#;

    println!("\nTesting: #ff0000");
    match compile(hex6) {
        Ok(css) => println!("SUCCESS: {}", css.trim()),
        Err(e) => println!("ERROR: {:?}", e),
    }

    // Test variable with hex
    let var_hex = r#"
@color: #333;
.test {
    color: @color;
}
"#;

    println!("\nTesting variable with hex: @color: #333");
    match compile(var_hex) {
        Ok(css) => println!("SUCCESS: {}", css.trim()),
        Err(e) => println!("ERROR: {:?}", e),
    }

    // Test percentage
    let percentage = r#"
.test {
    width: 20%;
}
"#;

    println!("\nTesting percentage: 20%");
    match compile(percentage) {
        Ok(css) => println!("SUCCESS: {}", css.trim()),
        Err(e) => println!("ERROR: {:?}", e),
    }
}
