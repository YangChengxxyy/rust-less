extern crate core;
#[macro_use]
extern crate pest_derive;

use std::fs;

use pest::Parser;
use selects::Selects;

mod parser;

mod selects;

mod utils;

fn main() {
    let file_s = fs::read_to_string("./src/test.less").expect("Not Found File!");
    let pairs = parser::LessParser::parse(parser::Rule::selects, &file_s).expect("Parser Error");
    let selects = Selects::new(pairs);
    fs::write("./src/test.css", selects.to_css()).expect("Write Error");
}
