use pest::iterators::Pair;
use crate::parser::Rule;
use crate::utils::{get_attr_and_variable, get_select_name};
use crate::selects::{Attr, Variable, ToCss};

#[derive(Debug)]
pub struct Select {
    pub select_name: String,
    pub select_value: String,
    pub span: (usize, usize),
    pub attr_list: Vec<Attr>,
    pub variable_list: Vec<Variable>,
    pub children: Box<Vec<Select>>,
    pub parent_select_names: Vec<String>,
    pub ancestor_variable_list: Vec<Variable>,
}

impl Select {
    pub fn new(pair: &Pair<Rule>, parents: Vec<String>, ancestor_variable_list: Vec<Variable>) -> Self {
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

    pub fn clear_value(&mut self) {
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

impl ToCss for Select {
    fn to_css(&self) -> String {
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
}