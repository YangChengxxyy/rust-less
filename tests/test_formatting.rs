//! 简化的格式化测试

use rust_less::compile;

#[test]
fn test_basic_margin_auto() {
    let less = ".test { margin: 0 auto; }";
    let result = compile(less).unwrap();

    println!("Input:  {}", less);
    println!("Output: {}", result.trim());

    // 基本检查 - 应该包含正确的格式
    assert!(result.contains("margin:"));
    assert!(result.contains("0"));
    assert!(result.contains("auto"));
}

#[test]
fn test_variable_with_auto() {
    let less = "@zero: 0; .test { margin: @zero auto; }";
    let result = compile(less).unwrap();

    println!("Input:  {}", less);
    println!("Output: {}", result.trim());

    // 检查变量是否正确替换
    assert!(result.contains("margin:"));
    assert!(result.contains("0"));
    assert!(result.contains("auto"));
}

#[test]
fn test_simple_calculation() {
    let less = ".test { width: 100px + 50px; }";
    let result = compile(less).unwrap();

    println!("Input:  {}", less);
    println!("Output: {}", result.trim());

    // 检查计算是否正确
    assert!(result.contains("width:"));
    assert!(result.contains("150px"));
}

#[test]
fn test_percentage_function() {
    let less = ".test { width: percentage(0.5); }";
    let result = compile(less).unwrap();

    println!("Input:  {}", less);
    println!("Output: {}", result.trim());

    // 检查百分比函数
    assert!(result.contains("width:"));
    assert!(result.contains("50%"));
}

#[test]
fn test_multiple_values() {
    let less = ".test { margin: 10px 20px 30px 40px; }";
    let result = compile(less).unwrap();

    println!("Input:  {}", less);
    println!("Output: {}", result.trim());

    // 检查多值是否正确分隔
    assert!(result.contains("margin:"));
    assert!(result.contains("10px"));
    assert!(result.contains("20px"));
    assert!(result.contains("30px"));
    assert!(result.contains("40px"));
}
