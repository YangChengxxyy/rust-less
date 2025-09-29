use rust_less::compile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"
.responsive {
    width: 100%;
    padding: 20px;

    @media (max-width: 768px) {
        width: 50%;
        padding: 10px;

        .inner {
            display: none;
        }

        &:hover {
            background: #eee;
        }
    }

    @media (min-width: 1200px) {
        width: 1200px;
        margin: 0 auto;
    }
}

.sidebar {
    width: 300px;

    @media (max-width: 768px) {
        width: 100%;
        position: fixed;
        top: 0;
        left: -100%;

        &.open {
            left: 0;
        }
    }
}
"#;

    println!("=== Input LESS ===");
    println!("{}", input);
    println!();

    match compile(input) {
        Ok(css) => {
            println!("=== Compiled CSS ===");
            println!("{}", css);
            println!();
            println!("✅ Compilation successful!");
        }
        Err(e) => {
            println!("❌ Compilation failed: {:?}", e);
            return Err(Box::new(e));
        }
    }

    Ok(())
}
