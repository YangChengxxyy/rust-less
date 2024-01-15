extern crate core;
#[macro_use]
extern crate pest_derive;

use std::fs;
use std::io::Read;

use pest::Parser;
use crate::parser::deal_single_pairs;

mod parser;

fn main() {
    let file_s = fs::read_to_string("./src/test.less").expect("Not Found File!");
    let paris = parser::LessParser::parse(parser::Rule::selects, &file_s).expect("Parser Error");
    deal_single_pairs(paris,"");
}