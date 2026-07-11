use std::collections::HashMap;

use pest::{
    Parser,
    iterators::{Pair, Pairs},
};

#[derive(pest_derive::Parser)]
#[grammar = "src/deserialize/file/file.pest"]
pub struct FileLexer;

impl FileLexer {
    pub fn raw_parse_str(input: &str) -> Result<Pairs<'_, Rule>, pest::error::Error<Rule>> {
        FileLexer::parse(Rule::file, input)
    }
}

#[derive(Debug)]
pub struct SiMaiFile(pub HashMap<String, String>);

impl TryFrom<&str> for SiMaiFile {
    type Error = &'static str;

    /// Try parse a `SiMai` format `&str` into `HashMap<String, String>`
    /// For example:
    /// ```maidata
    /// &name=Foo
    /// &inote_1=Bar
    /// E
    ///
    /// ```
    /// will be transformed into a hash map like:
    /// ```txt
    /// {"name" : "Foo"},
    /// {"inote_1" : "Bar\nE"}
    /// ```
    /// example:
    /// ```
    /// # use orbtiama_core::deserialize::file::lexer::SiMaiFile;
    /// # use std::collections::HashMap;
    /// let file=SiMaiFile::try_from("&name=Foo\n&inote_1=Bar\nE").expect("failed to parse").0;
    /// assert_eq!(file, HashMap::from([
    ///     ("name".to_string(), "Foo".to_string()),
    ///     ("inote_1".to_string(), "Bar\nE".to_string())
    /// ]));
    /// ```
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(SiMaiFile(
            FileLexer::raw_parse_str(value)
                .map_err(|_| "lexer failed")
                .and_then(|mut it| it.next().ok_or("root node not found"))
                .map(Pair::into_inner)
                .into_iter()
                .flat_map(|it| {
                    it.map(Pair::into_inner).map(|mut it| -> Option<_> {
                        (
                            String::from(it.find(|it| it.as_rule() == Rule::key)?.as_str().trim()),
                            String::from(
                                it.find(|it| it.as_rule() == Rule::content)?.as_str().trim(),
                            ),
                        )
                            .into()
                    })
                })
                .flatten()
                .fold(HashMap::new(), |mut map, it| {
                    map.insert(it.0, it.1);
                    map
                }),
        ))
    }
}
