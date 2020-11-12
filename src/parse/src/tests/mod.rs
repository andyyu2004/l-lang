use super::*;
use index::Idx;
use span::{Span, ROOT_FILE_IDX};

macro expect_parse_err($src:expr) {{
    let driver = ldriver::Driver::from_src($src);
    driver.parse().unwrap_err()
}}

#[test]
fn parse_redundant_visibility_qualifier() {
    let src = "pub impl T {}";
    expect_parse_err!(src);
}

#[test]
fn parse_fn_sig_missing_type_annotation() {
    let src = "fn f(x) { x }";
    expect_parse_err!(src);
}

macro parse_expr($src:expr) {{
    let driver = ldriver::Driver::from_src($src);
    driver.parse_expr().unwrap()
}}

macro fmt_expr($src:expr) {{
    let expr = parse_expr!($src);
    format!("{}", expr)
}}

#[test]
fn parse_deref() {
    parse_expr!("*x");
}

#[test]
fn parse_ref() {
    parse_expr!("&x");
}

#[test]
fn parse_chained_tuple_accesses() {
    // parse_expr!("x.1.1");
    // parse_expr!("x.1.1.1");
    parse_expr!("x.0.1.2.3.4.5.6");
}

#[test]
fn parse_assign() {
    let _expr = parse_expr!("x = y");
    let _expr = parse_expr!("x = y = 2");
}

#[test]
fn parse_nested_if() {
    let _expr = parse_expr!("if false { 5 } else if true { 6 } else { 7 }");
}

#[test]
fn parse_call_expr() {
    let _expr = parse_expr!("f(2,3,x)");
}

#[test]
fn parse_left_assoc_call_expr() {
    let expr = fmt_expr!("1(2)(3)(4)");
    assert_eq!(expr, "(((1 2) 3) 4)")
}

#[test]
fn test_parser_span() {
    let expr = parse_expr!("    3");
    assert_eq!(
        expr,
        box Expr::new(Span::new(ROOT_FILE_IDX, 4, 5), NodeId::new(0), ExprKind::Lit(Lit::Int(3)))
    );
}

#[test]
fn parse_empty_tuple() {
    let expr = parse_expr!("()");
    assert_eq!(
        expr,
        box Expr::new(Span::new(ROOT_FILE_IDX, 0, 2), NodeId::new(0), ExprKind::Tuple(vec![]))
    );
}

#[test]
fn parse_struct_expr() {
    let _expr = parse_expr!("SomeStruct { x: int, y: bool }");
}

#[test]
fn parse_tuple() {
    let expr = parse_expr!("(2, 3)");
    assert_eq!(
        expr,
        box Expr::new(
            Span::new(ROOT_FILE_IDX, 0, 6),
            NodeId::new(2),
            ExprKind::Tuple(vec![
                box Expr::new(
                    Span::new(ROOT_FILE_IDX, 1, 2),
                    NodeId::new(0),
                    ExprKind::Lit(Lit::Int(2))
                ),
                box Expr::new(
                    Span::new(ROOT_FILE_IDX, 4, 5),
                    NodeId::new(1),
                    ExprKind::Lit(Lit::Int(3))
                )
            ])
        )
    );
}

#[test]
fn parse_int_literal() {
    let expr = parse_expr!("2");
    assert_eq!(
        expr,
        box Expr::new(Span::new(ROOT_FILE_IDX, 0, 1), NodeId::new(0), ExprKind::Lit(Lit::Int(2)))
    );
}

#[test]
fn parse_simple_binary_expr() {
    let expr = parse_expr!("2 + 3");
    assert_eq!(
        expr,
        box Expr::new(
            Span::new(ROOT_FILE_IDX, 0, 5),
            NodeId::new(2),
            ExprKind::Bin(
                BinOp::Add,
                box Expr::new(
                    Span::new(ROOT_FILE_IDX, 0, 1),
                    NodeId::new(0),
                    ExprKind::Lit(Lit::Int(2))
                ),
                box Expr::new(
                    Span::new(ROOT_FILE_IDX, 4, 5),
                    NodeId::new(1),
                    ExprKind::Lit(Lit::Int(3))
                ),
            )
        )
    );
}

#[test]
fn parse_parameterless_lambda() {
    parse_expr!("fn () => 5");
}

#[test]
fn parse_lambda() {
    let _expr = parse_expr!("fn (x, y) => (2,3,4)");
}

#[test]
fn parse_typed_lambda() {
    let _expr = parse_expr!("fn (x: i64, y: f64) => (2,3,4)");
}

#[test]
fn parse_typed_lambda_with_ret_ty() {
    let _expr = parse_expr!("fn (x: i64, y: f64) -> (u64, u64, u64) => (2,3,4)");
}

#[test]
fn parse_precedence_expr() {
    let expr = parse_expr!("2 + 3 * 4");
    assert_eq!(
        expr,
        box Expr::new(
            Span::new(ROOT_FILE_IDX, 0, 9),
            NodeId::new(4),
            ExprKind::Bin(
                BinOp::Add,
                box Expr::new(
                    Span::new(ROOT_FILE_IDX, 0, 1),
                    NodeId::new(0),
                    ExprKind::Lit(Lit::Int(2))
                ),
                box Expr::new(
                    Span::new(ROOT_FILE_IDX, 4, 9),
                    NodeId::new(3),
                    ExprKind::Bin(
                        BinOp::Mul,
                        box Expr::new(
                            Span::new(ROOT_FILE_IDX, 4, 5),
                            NodeId::new(1),
                            ExprKind::Lit(Lit::Int(3))
                        ),
                        box Expr::new(
                            Span::new(ROOT_FILE_IDX, 8, 9),
                            NodeId::new(2),
                            ExprKind::Lit(Lit::Int(4))
                        ),
                    )
                ),
            )
        )
    );
}
