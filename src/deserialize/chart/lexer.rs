use pest::{Parser, iterators::Pairs};

#[derive(pest_derive::Parser)]
#[grammar = "src/deserialize/chart/simai.pest"]
pub struct ChartLexer;

impl ChartLexer {
    pub fn raw_parse_str(input: &str) -> Result<Pairs<'_, Rule>, pest::error::Error<Rule>> {
        ChartLexer::parse(Rule::chart, input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        deserialize::chart::lexer::{ChartLexer, Rule},
        function_test,
    };
    use expect_test::expect_file;
    use pest::iterators::Pair;

    fn test_rule(str: &str) -> String {
        let result = ChartLexer::raw_parse_str(str).unwrap();
        result
            .into_iter()
            .next()
            .unwrap()
            .into_inner()
            .fold(String::new(), |x, t| print_pair_(t, x, 0))
    }

    fn print_pair_(pair: Pair<'_, Rule>, string: String, tab: usize) -> String {
        let rule = pair.as_rule();
        let str = pair.as_str();
        let mut string = string + format!("{}{:?}: \"{}\"\n", " ".repeat(tab), rule, str).as_str();
        for it in pair.into_inner() {
            string = print_pair_(it, string, tab + 2);
        }
        string
    }

    macro_rules! slide_test {
        ($name:ident, $src:expr) => {
            function_test!($name, test_rule, $src);
        };
    }

    slide_test!(slide_test_01, "1-4[1:4]");
    slide_test!(slide_test_02, "1-4[1:4]");
    slide_test!(slide_test_03, "1-4[120#1:4]");
    slide_test!(slide_test_04, "1-4[0.5##1:4]");
    slide_test!(slide_test_05, "1-4[0.5##120#1:4]");
    slide_test!(slide_test_06, "1-4[0.5##1.4]");
    slide_test!(slide_test_07, "1-4-5[1:4]");
    slide_test!(slide_test_08, "1-4-5[120#1:4]");
    slide_test!(slide_test_09, "1-4-5[0.5##1:4]");
    slide_test!(slide_test_10, "1-4-5[0.5##120#1:4]");
    slide_test!(slide_test_11, "1-4-5[0.5##1.4]");
    slide_test!(slide_test_12, "1-4[1:4]-5[0.5##1.4]");
    slide_test!(slide_test_13, "1-4[#1.4]-5[0.5##1.4]");
    slide_test!(slide_test_14, "1-4[140#1:4]-5[0.5##1.4]b");
}
