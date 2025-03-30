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
}

impl Selects {
    pub fn new(pairs: Pairs<Rule>) -> Self {
        let mut children = vec![];
        let mut media_queries = vec![];
        let mut ancestor_variable_list = vec![];
        println!("{}",pairs.len());

        for pair in pairs {
            if pair.as_rule() == Rule::selects || pair.as_rule() == Rule::mediaQuery {
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::variable => {
                            ancestor_variable_list.insert(0, get_variable(pair));
                        }
                        Rule::select => {
                            children.push(Select::new(
                                &pair,
                                vec![],
                                ancestor_variable_list.clone(),
                            ));
                            ancestor_variable_list = vec![];
                        }
                        Rule::mediaQuery => {
                            media_queries
                                .push(MediaQuery::new(&pair, ancestor_variable_list.clone()));
                        }
                        _ => {}
                    }
                }
            }
        }

        Selects {
            children,
            media_queries,
        }
    }
}

impl ToCss for Selects {
    fn to_css(&self) -> String {
        let mut result = String::from("");

        for (i, child) in self.children.iter().enumerate() {
            result.push_str(&child.to_css());
            if i != 0 {
                result.push('\n');
            }
        }

        for media_query in &self.media_queries {
            result.push_str(&media_query.to_css());
            result.push('\n');
        }

        if !result.is_empty() && result.ends_with("\n\n") {
            result.pop();
            result.pop();
        } else if !result.is_empty() && result.ends_with('\n') {
            result.pop();
        }

        result
    }
}