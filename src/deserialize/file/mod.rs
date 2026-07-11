pub mod lexer;

#[cfg(test)]
mod test {
    use crate::deserialize::file::lexer::SiMaiFile;

    const METADATA_TEST: &str = r"
&title=Never Give Up!
&wholebpm=171
&artist=nanobii
&des=7.3連発華火
&shortid=11232
&genre=maimai
&cabinet=DX
&version=maimai DX Splash PLUS
&chartconverter=Neskol

    ";

    #[test]
    fn metadata_test() {
        let lexer = SiMaiFile::try_from(METADATA_TEST);
        let Ok(lex) = lexer else {
            panic!("failed to parse file");
        };
        assert_eq!(lex.0["title"], "Never Give Up!");
    }
}
