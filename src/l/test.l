enum Option {
    None,
    Some(int),
}

enum NodeOption {
    None,
    Some(&Node),
}

struct Node {
    val: int,
    next: NodeOption,
}

struct S {
    x: int,
    k: bool,
}

enum Either {
    Left(int),
    Right(int),
}

enum Expr {
    Int(int),
    Add(&Expr, &Expr)
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


