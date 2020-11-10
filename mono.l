
fn fst<T, U>((t, u): (T, U)) -> T { t }

/* fn snd<T, U>((t, u): (T, U)) -> U { fst((u, t)) } */

struct S<T, U> {
    t: T,
    u: U,
}

struct K<T, U> {
    s: S<T, U>
}

enum Option<T> {
    Some(T),
    None,
}



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
