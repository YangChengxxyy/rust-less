// 添加中文注释到 parser.rs 文件的代码中
#[derive(Parser, Default)]
#[grammar = "less.pest"]
pub struct LessParser {}
