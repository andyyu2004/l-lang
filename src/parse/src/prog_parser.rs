use super::*;
use ast::{Prog, P};

pub struct ProgParser;

impl<'a> Parse<'a> for ProgParser {
    type Output = P<Prog>;

    fn parse(&mut self, parser: &mut Parser<'a>) -> ParseResult<'a, Self::Output> {
        let mut items = vec![];
        while !parser.reached_eof() {
            items.push(ItemParser.parse(parser)?);
        }
        Ok(box Prog { items })
    }
}

#[cfg(test)]
/// rough tests that only checks whether things parse/don't parse as expected
mod test {
    macro_rules! parse {
        ($src:expr) => {{
            let driver = ldriver::Driver::from_src($src);
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
