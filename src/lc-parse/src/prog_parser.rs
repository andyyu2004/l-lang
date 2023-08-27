use super::*;
use lc_ast::{Ast};

pub struct AstParser;

impl<'a> Parse<'a> for AstParser {
    type Output = Ast;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let module = ModuleParser.parse(parser)?;
        Ok(Ast { module })
    }
}

#[cfg(test)]
/// rough tests that only checks whether things parse/don't parse as expected
mod test {
    macro_rules! parse {
        ($src:expr) => {{
            let driver = lc_driver::Driver::from_src($src);
            driver.parse()
        }};
    }

    #[test]
    fn parse_parameterless_empty_fn() {
        let _prog = parse!("fn test() {}");
    }

    #[test]
    fn parse_single_let_stmt() {
        let src = r#"
        fn test() {
            let x = 5;
        }
        "#;
        let _prog = parse!(src).unwrap();
    }

    #[test]
    fn parse_multiple_stmts() {
        let src = r#"
        fn test() {
            5;
            let y = 8;
        }
        "#;
        let _prog = parse!(src).unwrap();
    }

    #[test]
    fn parse_simple_var_path() {
        let src = r#"
        fn test() {
            let y = 8;
            y
        }
        "#;
        let _prog = parse!(src).unwrap();
    }

    #[test]
    fn parse_multi_segment_var_path() {
        let src = r#"
        fn test() {
            x::y::z
        }
        "#;
        let _prog = parse!(src).unwrap();
    }

    #[test]
    fn parse_missing_semi() {
        let src = r#"
        fn test() {
            8
            let y = 5;
        }
        "#;
        let _err = parse!(src).unwrap_err();
    }
}
