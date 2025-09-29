use rust_less::parser::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"@media (max-width: 768px) {
    width: 50%;
    .inner {
        display: none;
    }
}"#;

    println!("=== Testing Media Query Parsing ===");
    println!("Input:");
    println!("{}", input);
    println!();

    let mut parser = Parser::from_string(input.to_string())?;
    let stylesheet = parser.parse()?;

    println!("Parsed statements: {}", stylesheet.statements.len());

    for (i, statement) in stylesheet.statements.iter().enumerate() {
        println!("Statement {}: {:?}", i, statement);

        if let rust_less::ast::Statement::AtRule(at_rule) = statement {
            println!("  AtRule name: {}", at_rule.name);
            println!("  AtRule prelude: {:?}", at_rule.prelude);
            if let Some(block) = &at_rule.block {
                println!("  AtRule block ({} statements):", block.len());
                for (j, block_stmt) in block.iter().enumerate() {
                    println!("    Block statement {}: {:?}", j, block_stmt);
                }
            }
        }
    }

    Ok(())
}
