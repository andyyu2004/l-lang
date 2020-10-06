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
    /* "string"; */
    /* let expr = box Expr::Add( */
    /*     box Expr::Int(5), */
    /*     box Expr::Int(9), */
    /* ); */
    /* eval(expr) */

    let next = box Node {
	val: 22,
	next: NodeOption::None,
    };
    let node = box Node {
	val: 6,
	next: NodeOption::Some(next),
    };
    last(node);
    let x = box 5;
    *x
}

fn last(node: &Node) -> int {
    match node.next {
	NodeOption::Some(next) => last(next),
	NodeOption::None => node.val,
    }
}

/* fn eval(expr: &Expr) -> int { */
/*     match *expr { */
/*         Expr::Int(i) => i, */
/*         Expr::Add(l, r) => eval(l) + eval(r), */
/*     } */
/* } */


