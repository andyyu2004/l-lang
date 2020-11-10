use super::*;

#[test]
fn simple_conditionals() {
    let src = "fn main() -> int { if true { 20 } else { 30 } }";
    assert_eq!(llvm_exec!(src), 20);

    let src = "fn main() -> int { if false { 20 } else { 30 } }";
    assert_eq!(llvm_exec!(src), 30);
}

#[test]
fn simple_enum_match() {
    let src = r#"
    enum Option {
        Some(int),
        None,
    }

    fn main() -> int {
        let opt = Option::Some(9);
        match opt {
            Option::Some(x) => x,
            Option::None => 77,
        }
    }"#;

    assert_eq!(llvm_exec!(src), 9);
}

#[test]
fn simple_literal_match() {
    let src = r#"
    fn main() -> int {
        match 8 {
            8 => 50,
            _ => 33,
        }
    }"#;

    assert_eq!(llvm_exec!(src), 50);

    let src = r#"
    fn main() -> int {
        match 8 {
            30 => 50,
            _ => 34,
        }
    }"#;

    assert_eq!(llvm_exec!(src), 34);
}

#[test]
fn simple_expr_match() {
    let src = r#"
    enum Expr {
        Int(int),
        Add(&Expr, &Expr),
    }

    fn main() -> int {
        let expr = box Expr::Add(
            box Expr::Int(5),
            box Expr::Int(9),
        );
        eval(expr)
    }

    fn eval(expr: &Expr) -> int {
        match *expr {
            Expr::Int(i) => i,
            Expr::Add(l, r) => eval(l) + eval(r),
        }
    }
    "#;

    assert_eq!(llvm_exec!(src), 14);
}

#[test]
fn nested_match() {
    let src = r#"
    enum Option<T> {
        Some(T),
        None,
    }

    enum Either<L, R> {
        Left(L),
        Right(R),
    }

    fn main() -> int {
        let e = Either::Left(Option::Some(88));
        match e {
            Either::Left(opt) => match opt {
                Option::Some(i) => i,
                Option::None => 4,
            },
            Either::Right(x) => x,
        }
    }"#;

    assert_eq!(llvm_exec!(src), 88);
}
