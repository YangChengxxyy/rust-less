use rust_less::parser::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = "width: 50%;";

    let mut parser = Parser::from_string(input.to_string())?;
    let stylesheet = parser.parse()?;

    println!("=== Testing Declaration Parsing ===");
    println!("Input: {}", input);
    println!("Parsed statements: {}", stylesheet.statements.len());

    for (i, statement) in stylesheet.statements.iter().enumerate() {
        println!("Statement {}: {:?}", i, statement);
    }

    Ok(())
}
