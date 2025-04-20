extern crate core;
#[macro_use]
extern crate pest_derive;

mod parser;
mod selects;
mod test;
mod utils;

fn main() {
    println!("This is a LESS to CSS parser. To run the conversion tests, use 'cargo test'."); // 提示用户运行测试
    println!("Available tests include:"); // 提示可用的测试
    println!("  - nested_media_query_parse_test: Tests parsing of nested media queries"); // 嵌套媒体查询测试
    println!("  - basic_less_parse_test: Tests basic LESS file parsing"); // 基本 LESS 文件解析测试
}
