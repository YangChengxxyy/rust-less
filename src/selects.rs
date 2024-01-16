use pest::iterators::{Pair, Pairs};

use crate::parser::Rule;

#[derive(Debug)]
pub struct Selects {
    pub children: Vec<Select>,
}

#[derive(Debug)]
pub enum SelectEnum {
    ClassName,
    Label,
}

#[derive(Debug)]
pub struct Select {
    pub select_name: String,
    pub select_name_kind: SelectEnum,
    pub select_value: String,
    pub span: (usize, usize),
    pub children: Box<Vec<Select>>,
    pub parent_select_names: Vec<String>,
}

impl Selects {
    pub fn new(paris: Pairs<Rule>) -> Self {
        let mut children = vec![];
        for pair in paris {
            if pair.as_rule() == Rule::selects {
                for pair in pair.into_inner() {
                    if pair.as_rule() == Rule::select {
                        children.push(Select::new(&pair, vec![]));
                    }
                }
            }
        }

        let mut result = Selects { children };
        result.clear_value();
        return result;
    }

    fn clear_value(&mut self) {
        for child in &mut self.children {
            child.clear_value();
        }
    }

    pub fn to_css(&self) -> String {
        let mut result = String::from("");
        let mut i = 0;
        for child in &self.children {
            result.push_str(&child.to_css());
            if i != 0 {
                result.push('\n');
            }
            i = i + 1;
        }
        result
    }
}

impl Select {
    fn new(pair: &Pair<Rule>, parents: Vec<String>) -> Self {
        let rule = pair.as_rule();
        if rule != Rule::select {
            panic!("Not a select");
        } else {
            let result = get_select_name(&pair);
            let span = (pair.as_span().start(), pair.as_span().end());
            let child_pairs = pair.clone().into_inner();
            let mut children = vec![];

            if let Some(res) = result {
                let (select_name_kind, select_name) = res;
                for child_pair in child_pairs {
                    if child_pair.as_rule() == Rule::select {
                        let mut p = parents.clone();
                        p.push(select_name.clone());
                        children.push(Select::new(&child_pair, p));
                    }
                }
                let select = Select {
                    select_name,
                    select_name_kind,
                    span,
                    select_value: pair.as_span().as_str().to_string(),
                    children: Box::new(children),
                    parent_select_names: parents.clone(),
                };
                return select;
            } else {
                panic!("None, Cannot get select name")
            }
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

    fn to_css(&self) -> String {
        let mut result = String::from("");
        let mut parent_class_name = String::from("");
        let mut i = 0;
        for parent_name in &self.parent_select_names {
            if i != 0 {
                parent_class_name.push(' ');
            }
            parent_class_name.push_str(parent_name);
            i = i + 1;
        }
        result.push_str(&parent_class_name);
        result.push(' ');
        result.push_str(&self.select_value);
        result.push('\n');
        for child in self.children.as_slice() {
            result.push_str(&child.to_css());
        }
        return result;
    }
}

fn get_select_name(pair: &Pair<Rule>) -> Option<(SelectEnum, String)> {
    let rule = pair.as_rule();
    if rule != Rule::select {
        None
    } else {
        for child in pair.clone().into_inner() {
            if child.as_rule() == Rule::selectName {
                let str = child.as_span().as_str().to_string();
                let mut e = SelectEnum::ClassName;
                for child_child in child.into_inner() {
                    e = if child_child.as_rule() == Rule::className {
                        SelectEnum::ClassName
                    } else {
                        SelectEnum::Label
                    }
                }
                return Some((e, str));
            }
        }
        None
    }
}
