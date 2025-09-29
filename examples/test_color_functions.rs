use rust_less::compile;

fn main() {
    // Test simple color function parsing
    let less = r#"
@base-color: #333;

.test {
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
        }
        Err(e) => {
            println!("ERROR: {:?}", e);
        }
    }

    // Test individual components
    println!("\n{}", "=".repeat(50));
    println!("Testing individual color functions:");

    let simple_lighten = r#"
.test {
    color: lighten(#333, 20%);
}
"#;

    println!("\nTesting: lighten(#333, 20%)");
    match compile(simple_lighten) {
        Ok(css) => println!("SUCCESS: {}", css),
        Err(e) => println!("ERROR: {:?}", e),
    }

    let simple_darken = r#"
.test {
    color: darken(#333, 10%);
}
"#;

    println!("\nTesting: darken(#333, 10%)");
    match compile(simple_darken) {
        Ok(css) => println!("SUCCESS: {}", css),
        Err(e) => println!("ERROR: {:?}", e),
    }
}
