mod traits;
mod base;
mod select;
mod media_query;

pub use traits::*;
pub use base::*;
pub use select::*;
pub use media_query::*;

use pest::iterators::Pairs;
use crate::parser::Rule;
use crate::utils::get_variable;

#[derive(Debug)]
pub struct Selects {
    pub children: Vec<Select>, // 子选择器列表
    pub media_queries: Vec<MediaQuery>, // 媒体查询列表
    pub variables: Vec<Variable>, // 全局变量支持
}

impl Selects {
    pub fn new(pairs: Pairs<Rule>) -> Self {
        let mut children = vec![]; // 初始化子选择器列表
        let mut media_queries = vec![]; // 初始化媒体查询列表
        let mut variables = vec![]; // 初始化全局变量列表
        let mut ancestor_variable_list = vec![]; // 初始化祖先变量列表
        println!("{}",pairs.len()); // 打印 pairs 的长度

        for pair in pairs {
            if pair.as_rule() == Rule::selects {
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::variable => {
                            let var = get_variable(pair); // 获取变量
                            variables.push(var.clone()); // 将变量添加到全局变量列表
                            ancestor_variable_list.insert(0, var); // 插入到祖先变量列表的开头
                        }
                        Rule::select => {
                            children.push(Select::new(
                                &pair,
                                vec![],
                                ancestor_variable_list.clone(),
                            )); // 创建新的选择器并添加到子选择器列表
                        }
                        Rule::mediaQuery => {
                            media_queries.push(MediaQuery::new(&pair, ancestor_variable_list.clone())); // 创建新的媒体查询并添加到媒体查询列表
                        }
                        _ => {}
                    }
                }
            }
        }

        Selects {
            children, // 子选择器
            media_queries, // 媒体查询
            variables, // 全局变量
        }
    }
}

impl ToCss for Selects {
    fn to_css(&self) -> String {
        let mut result = String::new(); // 初始化结果字符串

        // 添加选择器
        for child in &self.children {
            result.push_str(&child.to_css()); // 将子选择器转换为 CSS 并添加到结果中
            result.push('\n'); // 添加换行符
        }

        // 添加媒体查询
        for media_query in &self.media_queries {
            result.push_str(&media_query.to_css()); // 将媒体查询转换为 CSS 并添加到结果中
            result.push('\n'); // 添加换行符
        }

        // 清理多余的换行符
        if !result.is_empty() && result.ends_with("\n\n") {
            result.pop(); // 移除最后一个换行符
            result.pop(); // 再次移除换行符
        } else if !result.is_empty() && result.ends_with('\n') {
            result.pop(); // 移除最后一个换行符
        }

        result // 返回结果字符串
    }
}