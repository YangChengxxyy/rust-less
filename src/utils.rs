use pest::iterators::Pair;

use crate::{
    parser::Rule,
    selects::{Attr, Variable},
};

pub fn get_select_name(pair: &Pair<Rule>) -> String {
    let rule = pair.as_rule();
    if rule != Rule::select {
        String::from("")
    } else {
        for child in pair.clone().into_inner() {
            if child.as_rule() == Rule::selectName {
                let str = child.as_span().as_str().to_string();
                return str;
            }
        }
        String::from("")
    }
}

pub fn get_attr_and_variable(pair: &Pair<Rule>) -> (Vec<Attr>, Vec<Variable>) {
    if pair.as_rule() != Rule::select {
        panic!("Not a select");
    }
    let mut attr_list = vec![];
    let mut variable_list = vec![];
    for child in pair.clone().into_inner() {
        if child.as_rule() == Rule::attr {
            attr_list.push(get_attr(child));
        } else if child.as_rule() == Rule::variable {
            variable_list.insert(0, get_variable(child));
        }
    }

    return (attr_list, variable_list);
}

pub fn get_attr(pair: Pair<Rule>) -> Attr {
    if pair.as_rule() != Rule::attr {
        panic!("Not a attr");
    }
    let mut name = String::from("");
    let mut value = String::from("");
    for child in pair.into_inner() {
        if child.as_rule() == Rule::attrName {
            name = child.as_span().as_str().to_string();
        }
        if child.as_rule() == Rule::attrValue {
            value = child.as_span().as_str().to_string();
        }
    }
    return Attr(name, value);
}

pub fn get_variable(pair: Pair<Rule>) -> Variable {
    if pair.as_rule() != Rule::variable {
        panic!("Not a variable");
    }
    let mut name = String::from("");
    let mut value = String::from("");
    for child in pair.into_inner() {
        if child.as_rule() == Rule::variableName {
            name = child.as_span().as_str().to_string();
        }
        if child.as_rule() == Rule::variableValue {
            value = child.as_span().as_str().to_string();
        }
    }
    return Variable(name, value);
}
