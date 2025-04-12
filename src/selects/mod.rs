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
    pub children: Vec<Select>,
    pub media_queries: Vec<MediaQuery>,
    pub variables: Vec<Variable>, // 添加全局变量支持
}

impl Selects {
    pub fn new(pairs: Pairs<Rule>) -> Self {
        let mut children = vec![];
        let mut media_queries = vec![];
        let mut variables = vec![]; // 初始化全局变量列表
        let mut ancestor_variable_list = vec![];
        println!("{}",pairs.len());

        for pair in pairs {
            if pair.as_rule() == Rule::selects {
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::variable => {
                            let var = get_variable(pair);
                            variables.push(var.clone());
                            ancestor_variable_list.insert(0, var);
                        }
                        Rule::select => {
                            children.push(Select::new(
                                &pair,
                                vec![],
                                ancestor_variable_list.clone(),
                            ));
                        }
                        Rule::mediaQuery => {
                            media_queries.push(MediaQuery::new(&pair, ancestor_variable_list.clone()));
                        }
                        _ => {}
                    }
                }
            }
        }

        Selects {
            children,
            media_queries,
            variables,
        }
    }
}

impl ToCss for Selects {
    fn to_css(&self) -> String {
        let mut result = String::new();

        // 添加选择器
        for child in &self.children {
            result.push_str(&child.to_css());
            result.push('\n');
        }

        // 添加媒体查询
        for media_query in &self.media_queries {
            result.push_str(&media_query.to_css());
            result.push('\n');
        }

        // 清理多余的换行符
        if !result.is_empty() && result.ends_with("\n\n") {
            result.pop();
            result.pop();
        } else if !result.is_empty() && result.ends_with('\n') {
            result.pop();
        }

        result
    }
}