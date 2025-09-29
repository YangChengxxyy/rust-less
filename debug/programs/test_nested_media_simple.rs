use rust_less::parser::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#".responsive {
    width: 100%;

    @media (max-width: 768px) {
        width: 50%;
        .inner {
            display: none;
        }
    }
}"#;

    println!("=== Testing Nested Media Query Parsing ===");
    println!("Input:");
    println!("{}", input);
    println!();

    let mut parser = Parser::from_string(input.to_string())?;
    let stylesheet = parser.parse()?;

    println!("Parsed statements: {}", stylesheet.statements.len());

    for (i, statement) in stylesheet.statements.iter().enumerate() {
        println!("Statement {}: {:?}", i, statement);

        if let rust_less::ast::Statement::Rule(rule) = statement {
            println!("  Rule selectors: {:?}", rule.selectors);
            println!("  Rule declarations: {:?}", rule.declarations);
            println!("  Nested rules ({}):", rule.nested_rules.len());
            for (j, nested) in rule.nested_rules.iter().enumerate() {
                println!("    Nested {}: {:?}", j, nested);
                if let rust_less::ast::Statement::AtRule(at_rule) = nested {
                    println!("      AtRule name: {}", at_rule.name);
                    println!("      AtRule prelude: {:?}", at_rule.prelude);
                    if let Some(block) = &at_rule.block {
                        println!("      AtRule block ({} statements):", block.len());
                        for (k, block_stmt) in block.iter().enumerate() {
                            println!("        Block statement {}: {:?}", k, block_stmt);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
