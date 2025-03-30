use pest::iterators::Pair;
use crate::parser::Rule;
use crate::utils::get_variable;
use crate::selects::{Variable, Select, ToCss};

#[derive(Debug)]
pub struct MediaQuery {
    pub query_expression: String,
    pub children: Vec<Select>,
    pub variable_list: Vec<Variable>,
    pub span: (usize, usize),
}

impl MediaQuery {
    pub fn new(pair: &Pair<Rule>, ancestor_variable_list: Vec<Variable>) -> Self {
        let mut query_expression = String::new();
        let mut children = vec![];
        let mut variable_list = vec![];
        let span = (pair.as_span().start(), pair.as_span().end());

        for inner_pair in pair.clone().into_inner() {
            match inner_pair.as_rule() {
                Rule::mediaQueryExpression => {
                    query_expression = inner_pair.as_span().as_str().to_string();
                }
                Rule::select => {
                    let mut new_ancestor_variable_list = ancestor_variable_list.clone();
                    for item in variable_list.clone() {
                        new_ancestor_variable_list.insert(0, item);
                    }
                    children.push(Select::new(&inner_pair, vec![], new_ancestor_variable_list));
                }
                Rule::variable => {
                    variable_list.insert(0, get_variable(inner_pair));
                }
                _ => {}
            }
        }

        MediaQuery {
            query_expression,
            children,
            variable_list,
            span,
        }
    }
}

impl ToCss for MediaQuery {
    fn to_css(&self) -> String {
        let mut result = String::from("@media ");
        result.push_str(&self.query_expression);
        result.push_str(" {\n");

        for child in &self.children {
            result.push_str(&child.to_css());
        }

        result.push_str("}\n");
        result
    }
}