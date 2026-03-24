use xoryzayin::test_from_files;

pub mod chart;
pub mod file;

#[test_from_files("maidata.txt")]
fn test_level_data(file: String) {
    use crate::deserialize::{
        chart::{lexer::ChartLexer, parser::ChartParser},
        file::lexer::SiMaiFile,
    };
    let chart = SiMaiFile::try_from(file.as_ref()).expect("file parse failed");
    chart
        .0
        .into_iter()
        .filter(|it| it.0.starts_with("inote_"))
        .map(|(k, v)| {
            (
                k.clone(),
                ChartLexer::raw_parse_str(&v)
                    .map(ChartParser::parse_chart)
                    .unwrap_or_else(|e| panic!("failed to parse: {:?}: {:?}", k.clone(), e)),
            )
        })
        .for_each(|_| ());
}
