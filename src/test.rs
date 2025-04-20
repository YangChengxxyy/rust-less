#[cfg(test)] // 仅在测试模式下编译
mod tests {
    use crate::parser::{self, LessParser, Rule};
    use pest::Parser;

    #[test] // 定义一个测试函数
    fn media_query_test() {
        let s = "@media screen and (min-width: 600px) {
  body {
    background-color: red;
  }
}"; // 定义测试的媒体查询字符串
        let pairs = LessParser::parse(Rule::mediaQuery, s); // 修正：使用mediaQuery
        println!("{:#?}", pairs); // 打印解析结果
        assert!(pairs.is_ok()); // 验证解析是否成功
    }

    #[test]
    fn class_name_test() {
        let s = ".example-class { color: blue; }"; // 定义测试的类名选择器字符串
        let pairs = LessParser::parse(Rule::select, s); // 解析类名选择器
        println!("{:#?}", pairs); // 打印解析结果
        assert!(pairs.is_ok()); // 验证解析是否成功
    }

    #[test]
    fn id_test() {
        let s = "#example-id { margin: 10px; }"; // 定义测试的 ID 选择器字符串
        let pairs = parser::LessParser::parse(parser::Rule::select, s); // 解析 ID 选择器
        println!("{:#?}", pairs); // 打印解析结果
        assert!(pairs.is_ok()); // 验证解析是否成功
    }

    #[test]
    fn nested_selectors_test() {
        let s = ".parent { .child { padding: 5px; } }"; // 定义测试的嵌套选择器字符串
        let pairs = parser::LessParser::parse(parser::Rule::select, s); // 解析嵌套选择器
        println!("{:#?}", pairs); // 打印解析结果
        assert!(pairs.is_ok()); // 验证解析是否成功
    }

    #[test]
    fn pseudo_class_test() {
        let s = "a:hover { text-decoration: underline; }"; // 定义测试的伪类选择器字符串
        let pairs = parser::LessParser::parse(parser::Rule::select, s); // 解析伪类选择器
        println!("{:#?}", pairs); // 打印解析结果
        assert!(pairs.is_ok()); // 验证解析是否成功
    }

    #[test]
    fn pseudo_element_test() {
        let s = "p::after { content: 'Hello'; }"; // 定义测试的伪元素选择器字符串
        let pairs = parser::LessParser::parse(parser::Rule::select, s); // 解析伪元素选择器
        println!("{:#?}", pairs); // 打印解析结果
        assert!(pairs.is_ok()); // 验证解析是否成功
    }

    #[test]
    fn attr_direct_test() {
        let s = "color: blue;"; // 定义测试的直接属性字符串
        let pairs = parser::LessParser::parse(parser::Rule::attr, s); // 解析直接属性
        println!("{:#?}", pairs); // 打印解析结果
        assert!(pairs.is_ok()); // 验证解析是否成功
    }

    #[test]
    fn attr_with_spaces_test() {
        let s = "margin: 10px 20px 5px 15px;"; // 定义测试的带空格属性字符串
        let pairs = parser::LessParser::parse(parser::Rule::attr, s); // 解析带空格属性
        println!("{:#?}", pairs); // 打印解析结果
        assert!(pairs.is_ok()); // 验证解析是否成功
    }

    #[test]
    fn attr_with_newline_test() {
        let s = "background: url('image.jpg')\n                  no-repeat;"; // 定义测试的带换行符属性字符串
        let pairs = parser::LessParser::parse(parser::Rule::attr, s); // 解析带换行符属性
        println!("{:#?}", pairs); // 打印解析结果
        assert!(pairs.is_ok()); // 验证解析是否成功
    }

    #[test]
    fn background_shorthand_test() {
        let s = "body { background: red; }"; // 定义测试的背景简写属性字符串
        let pairs = parser::LessParser::parse(parser::Rule::select, s); // 解析背景简写属性
        println!("{:#?}", pairs); // 打印解析结果
        assert!(pairs.is_ok()); // 验证解析是否成功
    }

    #[test]
    fn variable_direct_test() {
        let s = "@primary-color: #333;"; // 定义测试的直接变量字符串
        let pairs = parser::LessParser::parse(parser::Rule::variable, s); // 解析直接变量
        println!("{:#?}", pairs); // 打印解析结果
        assert!(pairs.is_ok()); // 验证解析是否成功
    }

    #[test]
    fn variable_in_selector_test() {
        let s = "body { @theme-color: blue; color: @theme-color; }"; // 定义测试的选择器中变量字符串
        let pairs = parser::LessParser::parse(parser::Rule::select, s); // 解析选择器中变量
        println!("{:#?}", pairs); // 打印解析结果
        assert!(pairs.is_ok()); // 验证解析是否成功
    }

    #[test]
    fn variable_with_complex_value_test() {
        let s = "@box-shadow: 0 1px 2px rgba(0,0,0,0.2);"; // 定义测试的复杂值变量字符串
        let pairs = parser::LessParser::parse(parser::Rule::variable, s); // 解析复杂值变量
        println!("{:#?}", pairs); // 打印解析结果
        assert!(pairs.is_ok()); // 验证解析是否成功
    }

    #[test]
    fn nested_media_query_parse_test() {
        let nested_media_file = std::fs::read_to_string("./src/test_nested_media.less")
            .expect("Not Found Nested Media Test File!"); // 读取嵌套媒体查询测试文件

        match crate::parse_less(&nested_media_file) {
            Ok(css_content) => {
                // 验证生成的CSS内容不为空
                assert!(!css_content.is_empty(), "生成的CSS内容不应为空");

                // 可选：将结果写入文件进行测试
                std::fs::write("./src/test_nested_media.css", css_content)
                    .expect("Write Error for Nested Media CSS");
            }
            Err(e) => {
                println!("解析失败: {:?}", e);
                assert!(false, "嵌套媒体查询解析应当成功");
            }
        }
    }

    #[test]
    fn basic_less_parse_test() {
        match crate::parse_less_file("./src/test.less") {
            Ok(css_content) => {
                // 验证生成的CSS内容不为空
                assert!(!css_content.is_empty(), "生成的CSS内容不应为空");

                // 可选：将结果写入文件进行测试
                std::fs::write("./src/test.css", css_content).expect("Write Error");
            }
            Err(e) => {
                println!("解析失败: {:?}", e);
                assert!(false, "基本LESS解析应当成功");
            }
        }
    }

    #[test]
    fn library_api_test() {
        let less = ".container { @width: 80%; width: @width; .header { color: blue; } }";
        match crate::parse_less(less) {
            Ok(css) => {
                assert!(css.contains(".container"));
                assert!(css.contains("width: 80%"));
                assert!(css.contains(".container .header"));
            }
            Err(e) => {
                panic!("库API测试失败: {}", e);
            }
        }
    }
}
