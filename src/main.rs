#[macro_use]
extern crate pest_derive;

use std::fs;
use std::io::Read;
use pest::iterators::Pairs;

use pest::Parser;

mod ast;
mod parser;
mod driver;
mod interpreter;

#[derive(Parser, Default)]
#[grammar = "less.pest"]
struct LessParser;

fn main() {
    let file_s = fs::read_to_string("./src/test.less").expect("Not Found File!");
    let temp = LessParser::parse(Rule::selects, &file_s).expect("Parser Error");

    println!("{:#?}", temp);
}