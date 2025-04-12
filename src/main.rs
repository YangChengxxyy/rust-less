extern crate core;
#[macro_use]
extern crate pest_derive;

use std::fs;

use crate::selects::ToCss;
use pest::Parser;
use selects::Selects;

mod parser;

mod selects;
mod test;
mod utils;

fn main() {
    // 解析嵌套媒体查询测试文件
    let nested_media_file = fs::read_to_string("./src/test_nested_media.less").expect("Not Found Nested Media Test File!");
    println!("解析嵌套媒体查询文件...");
    
    // 添加调试信息
    match parser::LessParser::parse(parser::Rule::selects, &nested_media_file) {
        Ok(pairs) => {
            println!("成功解析嵌套媒体查询!");
            
            // 打印解析结果的第一层结构
            println!("解析结果的顶层结构:");
            let pairs_clone = pairs.clone();
            for pair in pairs_clone {
                println!("Rule: {:?}, Text: {}", pair.as_rule(), pair.as_str());
                
                // 打印第二层结构
                for inner_pair in pair.into_inner() {
                    println!("  Inner Rule: {:?}, Text: {}", inner_pair.as_rule(), inner_pair.as_str().lines().next().unwrap_or(""));
                }
            }
            
            let nested_selects = Selects::new(pairs);
            let css_content = nested_selects.to_css();
            println!("生成的CSS内容长度: {}", css_content.len());
            if css_content.is_empty() {
                println!("警告: 生成的CSS内容为空!");
            } else {
                println!("CSS内容预览: {}", css_content.lines().take(5).collect::<Vec<_>>().join("\n"));
            }
            
            fs::write("./src/test_nested_media.css", css_content).expect("Write Error for Nested Media CSS");
            println!("已生成嵌套媒体查询CSS文件");
        },
        Err(e) => {
            println!("解析失败: {:?}", e);
        }
    }
    
    // 原始测试文件解析
    let file_s = fs::read_to_string("./src/test.less").expect("Not Found File!");
    let pairs = parser::LessParser::parse(parser::Rule::selects, &file_s).expect("Parser Error");
    let selects = Selects::new(pairs);
    fs::write("./src/test.css", selects.to_css()).expect("Write Error");
    println!("已生成原始测试CSS文件");
}
