use rust_less::compile;

fn main() {
    println!("Testing comment preservation...\n");

    let less_with_comments = r#"
/* This is a block comment */
.test {
    // This is a line comment
    color: red; /* inline comment */
    background: blue;
}

/* Another block comment */
.another {
    margin: 10px;
}
"#;

    println!("LESS input:");
    println!("{}", less_with_comments);
    println!("\n{}", "=".repeat(50));

    match compile(less_with_comments) {
        Ok(css) => {
            println!("SUCCESS! Compiled CSS:");
            println!("{}", css);

            println!("\n{}", "=".repeat(50));
            println!("Comment preservation check:");

            let checks = vec![
                (
                    "/* This is a block comment */",
                    css.contains("/* This is a block comment */"),
                ),
                ("/* inline comment */", css.contains("/* inline comment */")),
                (
                    "/* Another block comment */",
                    css.contains("/* Another block comment */"),
                ),
            ];

            for (comment, found) in checks {
                println!(
                    "  {} -> {}",
                    comment,
                    if found { "✅ FOUND" } else { "❌ MISSING" }
                );
            }
        }
        Err(e) => {
            println!("ERROR: {:?}", e);
        }
    }

    // Test different comment styles
    println!("\n{}", "=".repeat(50));
    println!("Testing different comment styles:");

    let comment_styles = vec![
        ("Block comment", "/* comment */ .test { color: red; }"),
        ("Line comment", "// comment\n.test { color: red; }"),
        ("Multiple block", "/* a */ /* b */ .test { color: red; }"),
        ("Nested style", ".test { /* comment */ color: red; }"),
    ];

    for (name, less) in comment_styles {
        println!("\n{}: {}", name, less);
        match compile(less) {
            Ok(css) => println!("  Result: {}", css.trim()),
            Err(e) => println!("  Error: {:?}", e),
        }
    }
}
