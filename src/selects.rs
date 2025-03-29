use pest::iterators::{Pair, Pairs};

use crate::{
    parser::Rule,
    utils::{get_attr_and_variable, get_select_name, get_variable},
};

#[derive(Debug)]
pub struct Selects {
    pub children: Vec<Select>,
    pub media_queries: Vec<MediaQuery>,
}

pub trait ToCss {
    fn to_css(&self) -> String;
}

/** 属性 */
#[derive(Debug)]
pub struct Attr(pub String, pub String);

impl Clone for Attr {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

/** 变量 */
#[derive(Debug)]
pub struct Variable(pub String, pub String);

impl Clone for Variable {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

#[derive(Debug)]
pub struct MediaQuery {
    pub query_expression: String,
    pub children: Vec<dyn ToCss>,
    pub variable_list: Vec<Variable>,
    pub span: (usize, usize),
}

#[derive(Debug)]
/** 选择器 */
pub struct Select {
    /** 选择器名 */
    pub select_name: String,
    /** 选择器完整的值 */
    pub select_value: String,
    /** 在源文件的位置 */
    pub span: (usize, usize),
    /** 属性列表 */
    pub attr_list: Vec<Attr>,
    /** 变量列表 */
    pub variable_list: Vec<Variable>,
    /** 内部的选择器 */
    pub children: Box<Vec<dyn ToCss>>,
    /** 父节点的选择器名 */
    pub parent_select_names: Vec<String>,
    /** 祖先节点的变量池 */
    pub ancestor_variable_list: Vec<Variable>,
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

impl Selects {
    pub fn new(pairs: Pairs<Rule>) -> Self {
        let mut children = vec![];
        let mut media_queries = vec![];
        let mut ancestor_variable_list = vec![];

        for pair in pairs {
            if pair.as_rule() == Rule::selects {
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
    // 修改 to_css 方法，包含媒体查询输出
    fn to_css(&self) -> String {
        let mut result = String::from("");

        // 处理普通选择器
        for (i, child) in self.children.iter().enumerate() {
            result.push_str(&child.to_css());
            if i != 0 {
                result.push('\n');
            }
        }

        // 处理媒体查询
        for media_query in &self.media_queries {
            result.push_str(&media_query.to_css());
            result.push('\n');
        }

        // 清理多余换行符
        if !result.is_empty() && result.ends_with("\n\n") {
            result.pop();
            result.pop();
        } else if !result.is_empty() && result.ends_with('\n') {
            result.pop();
        }

        result
    }
}

impl Select {
    fn new(pair: &Pair<Rule>, parents: Vec<String>, ancestor_variable_list: Vec<Variable>) -> Self {
        let rule = pair.as_rule();
        if rule != Rule::select {
            panic!("Not a select");
        } else {
            let select_name = get_select_name(&pair);

            let span = (pair.as_span().start(), pair.as_span().end());

            let child_pairs = pair.clone().into_inner();

            let mut children = vec![];

            let (attr_list, variable_list) = get_attr_and_variable(pair);

            for child_pair in child_pairs {
                if child_pair.as_rule() == Rule::select {
                    let mut p = parents.clone();
                    p.push(select_name.clone());

                    let mut new_ancestor_variable_list = vec![];
                    for item in ancestor_variable_list.clone() {
                        new_ancestor_variable_list.insert(0, item);
                    }

                    for item in variable_list.clone() {
                        new_ancestor_variable_list.insert(0, item);
                    }

                    children.push(Select::new(&child_pair, p, new_ancestor_variable_list));
                }
            }

            let select = Select {
                select_name,
                span,
                select_value: pair.as_span().as_str().to_string(),
                children: Box::new(children),
                parent_select_names: parents.clone(),
                variable_list,
                attr_list,
                ancestor_variable_list,
            };
            return select;
        }
    }

    fn clear_value(&mut self) {
        let mut need_removes: Vec<(usize, usize)> = vec![];
        for child in self.children.as_mut_slice() {
            let remove = (child.span.0 - self.span.0, child.span.1 - self.span.0);
            need_removes.push(remove);
            child.clear_value();
        }

        let mut diff = 0;
        for (mut start, mut end) in need_removes {
            start = start - diff;
            end = end - diff;
            diff = diff + end - start;
            self.select_value.replace_range(start..end, "");
        }
    }

    pub fn to_css(&self) -> String {
        let mut result = String::from("");

        let mut full_class_name = String::from("");
        let parent_class_name = self.parent_select_names.join(" ");
        if self.select_name.contains('&') && &self.select_name != "&" {
            full_class_name.push_str(&self.select_name.replace("&", &parent_class_name));
        } else {
            full_class_name.push_str(&parent_class_name);
            if self.parent_select_names.len() > 0 {
                full_class_name.push(' ');
            }
            full_class_name.push_str(&self.select_name);
        }

        let mut value = String::from("");

        value.push_str(&full_class_name);
        value.push(' ');
        value.push('{');
        value.push('\n');

        for attr in &self.attr_list {
            let mut attr_s = String::from("  ");
            attr_s.push_str(&attr.0);
            attr_s.push_str(": ");
            if let Some(res) = self.find_variable(&attr.1) {
                attr_s.push_str(&res);
            } else {
                attr_s.push_str(&attr.1);
            }
            attr_s.push_str(";\n");

            value.push_str(&attr_s);
        }

        value.push_str("}\n");

        result.push_str(&value);

        result.push('\n');

        for child in self.children.as_slice() {
            result.push_str(&child.to_css());
        }

        return result;
    }

    pub fn find_variable(&self, name: &str) -> Option<String> {
        let variable_list = &self.variable_list;
        let ancestor_variable_list = &self.ancestor_variable_list;

        let result = variable_list.iter().find(|&item| {
            if item.0 == *name {
                return true;
            }
            return false;
        });

        if let Some(res) = result {
            return Some(res.1.clone());
        } else {
            let result = ancestor_variable_list.iter().find(|&item| {
                if item.0 == *name {
                    return true;
                }
                return false;
            });
            if let Some(res) = result {
                Some(res.1.clone())
            } else {
                None
            }
        }
    }
}
