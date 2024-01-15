use pest::iterators::{Pair, Pairs};

#[derive(Parser, Default)]
#[grammar = "less.pest"]
pub struct LessParser {}

pub fn deal_single_pairs(pairs: Pairs<Rule>, parent: &str) {
    for pair in pairs {
        let rule = pair.as_rule();
        match rule {
            Rule::selectName => {}
            Rule::select => {}
            Rule::selects => {}
            _ => {}
        }

        deal_single_pairs(pair.into_inner(), parent);
    }
}