use pest::Parser;
use pest::iterators::Pair;
use crate::selects::ToCss;
use crate::selects::select::Select;
use crate::selects::media_query::MediaQuery;

#[derive(Parser)]
#[grammar = "less.pest"]
pub struct LessParser;

/// 表示解析后的LESS抽象语法树
pub enum LessAst {
    Select(Select),
    MediaQuery(MediaQuery),
}

impl ToCss for LessAst {
    fn to_css(&self) -> String {
        match self {
            LessAst::Select(select) => select.to_css(),
            LessAst::MediaQuery(media_query) => media_query.to_css(),
        }
    }
}

/// 解析LESS字符串，返回抽象语法树
///
/// # 参数
///
/// * `source` - LESS源代码字符串
///
/// # 返回值
///
/// 成功时返回`Ok(LessAst)`，失败时返回`Err(String)`
pub fn parse(source: &str) -> Result<LessAst, String> {
    // 修正：使用selects而不是less
    match LessParser::parse(Rule::selects, source) {
        Ok(mut pairs) => {
            let pair = pairs.next().unwrap();
            
            let inner_pairs: Vec<Pair<Rule>> = pair.into_inner().collect();
            
            for inner_pair in inner_pairs {
                match inner_pair.as_rule() {
                    Rule::select => {
                        let select = Select::new(&inner_pair, vec![], vec![]);
                        return Ok(LessAst::Select(select));
                    },
                    // 修正：使用mediaQuery而不是media_query，并且传递空的变量列表作为第二个参数
                    Rule::mediaQuery => {
                        let media_query = MediaQuery::new(&inner_pair, vec![]);
                        return Ok(LessAst::MediaQuery(media_query));
                    },
                    _ => continue,
                }
            }
            
            Err("No valid LESS content found".to_string())
        },
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}
