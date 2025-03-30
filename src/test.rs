#[cfg(test)]
mod tests {
    use pest::Parser;
    use crate::parser;


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

    #[test]
    fn class_name_test() {
        let s = ".example-class { color: blue; }";
        let pairs = parser::LessParser::parse(parser::Rule::select, s);
        println!("{:#?}", pairs);
        assert!(pairs.is_ok());
    }

    #[test]
    fn id_test() {
        let s = "#example-id { margin: 10px; }";
        let pairs = parser::LessParser::parse(parser::Rule::select, s);
        println!("{:#?}", pairs);
        assert!(pairs.is_ok());
    }

    #[test]
    fn nested_selectors_test() {
        let s = ".parent { .child { padding: 5px; } }";
        let pairs = parser::LessParser::parse(parser::Rule::select, s);
        println!("{:#?}", pairs);
        assert!(pairs.is_ok());
    }

    #[test]
    fn pseudo_class_test() {
        let s = "a:hover { text-decoration: underline; }";
        let pairs = parser::LessParser::parse(parser::Rule::select, s);
        println!("{:#?}", pairs);
        assert!(pairs.is_ok());
    }

    #[test]
    fn pseudo_element_test() {
        let s = "p::after { content: 'Hello'; }";
        let pairs = parser::LessParser::parse(parser::Rule::select, s);
        println!("{:#?}", pairs);
        assert!(pairs.is_ok());
    }

    #[test]
    fn attr_direct_test() {
        let s = "color: blue;";
        let pairs = parser::LessParser::parse(parser::Rule::attr, s);
        println!("{:#?}", pairs);
        assert!(pairs.is_ok());
    }

    #[test]
    fn attr_with_spaces_test() {
        let s = "margin: 10px 20px 5px 15px;";
        let pairs = parser::LessParser::parse(parser::Rule::attr, s);
        println!("{:#?}", pairs);
        assert!(pairs.is_ok());
    }

    #[test]
    fn attr_with_newline_test() {
        let s = "background: url('image.jpg')\n                  no-repeat;";
        let pairs = parser::LessParser::parse(parser::Rule::attr, s);
        println!("{:#?}", pairs);
        assert!(pairs.is_ok());
    }

    #[test]
    fn background_shorthand_test() {
        let s = "body { background: red; }";
        let pairs = parser::LessParser::parse(parser::Rule::select, s);
        println!("{:#?}", pairs);
        assert!(pairs.is_ok());
    }

    #[test]
    fn variable_direct_test() {
        let s = "@primary-color: #333;";
        let pairs = parser::LessParser::parse(parser::Rule::variable, s);
        println!("{:#?}", pairs);
        assert!(pairs.is_ok());
    }

    #[test]
    fn variable_in_selector_test() {
        let s = "body { @theme-color: blue; color: @theme-color; }";
        let pairs = parser::LessParser::parse(parser::Rule::select, s);
        println!("{:#?}", pairs);
        assert!(pairs.is_ok());
    }

    #[test]
    fn variable_with_complex_value_test() {
        let s = "@box-shadow: 0 1px 2px rgba(0,0,0,0.2);";
        let pairs = parser::LessParser::parse(parser::Rule::variable, s);
        println!("{:#?}", pairs);
        assert!(pairs.is_ok());
    }
}