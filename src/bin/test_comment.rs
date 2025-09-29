use rust_less::compile;

fn main() {
    let less = r#"/* This is a block comment */
.test {
    color: red;
}
"#;

    match compile(less) {
        Ok(css) => {
            println!("=== CSS OUTPUT ===");
            println!("{}", css);
            println!("=== END ===");
            println!(
                "Contains block comment: {}",
                css.contains("/* This is a block comment */")
            );
        }
        Err(e) => println!("Error: {}", e),
    }
}
