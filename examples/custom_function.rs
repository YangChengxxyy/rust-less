use rust_less::ast::{Expression, Position};
use rust_less::{Compiler, Result};

fn main() -> Result<()> {
    let mut compiler = Compiler::new();
    compiler.register_function(
        "double",
        Box::new(|args: &[Expression], pos: &Position| match &args[0] {
            Expression::Number { value, unit, .. } => Ok(match unit {
                Some(u) => Expression::number_with_unit(value * 2.0, u.clone(), pos.clone()),
                None => Expression::number(value * 2.0, pos.clone()),
            }),
            _ => Ok(args[0].clone()),
        }),
    );
    let css = compiler.compile(".x { width: double(21px); }")?;
    assert!(css.contains("width: 42px"), "Got: {}", css);
    println!("OK: {}", css.trim());
    Ok(())
}
