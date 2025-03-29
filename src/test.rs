#[cfg(test)]
mod tests {
    use pest::Parser;
    use crate::parser;
    use super::*;

    #[test]
    fn media_query_test() {
        let s = "@media  screen and (min-width: 600px) {
  body {
    background-color: red;
  }
}";
        let pairs = parser::LessParser::parse(parser::Rule::mediaQuery, s);
        println!("{:#?}", pairs);
        assert!(pairs.is_ok());
    }
}
