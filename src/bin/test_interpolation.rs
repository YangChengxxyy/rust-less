use rust_less::compile;

fn main() {
    // Test 1: Simple variable interpolation in selector
    let less1 = r#"
@selector: "header";

.@{selector} {
    color: red;
}
"#;

    println!("=== Test 1: Selector Interpolation ===");
    match compile(less1) {
        Ok(css) => {
            println!("CSS Output:");
            println!("{}", css);
            println!("Expected: .header");
            println!("Contains '.header': {}", css.contains(".header"));
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test 2: Simple variable interpolation in property value
    let less2 = r#"
@base: "images";

.test {
    background: @{base};
}
"#;

    println!("\n=== Test 2: Property Value Interpolation ===");
    match compile(less2) {
        Ok(css) => {
            println!("CSS Output:");
            println!("{}", css);
            println!("Expected: images");
            println!("Contains 'images': {}", css.contains("images"));
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test 3: Combined test without url() function
    let less3 = r#"
@selector: "button";
@color: "blue";

.@{selector} {
    color: @{color};
}
"#;

    println!("\n=== Test 3: Combined Interpolation ===");
    match compile(less3) {
        Ok(css) => {
            println!("CSS Output:");
            println!("{}", css);
            println!("Expected: .button with color: blue");
            println!("Contains '.button': {}", css.contains(".button"));
            println!("Contains 'blue': {}", css.contains("blue"));
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test 4: Original test case with URL
    let less4 = r#"
@base-url: "images";
@selector: "header";

.@{selector} {
    background: url("@{base-url}/bg.png");
}
"#;

    println!("\n=== Test 4: Original URL Interpolation Test ===");
    match compile(less4) {
        Ok(css) => {
            println!("CSS Output:");
            println!("{}", css);
            println!("Expected: .header with url(\"images/bg.png\")");
            println!("Contains '.header': {}", css.contains(".header"));
            println!("Contains 'images': {}", css.contains("images"));
        }
        Err(e) => println!("Error: {}", e),
    }
}
