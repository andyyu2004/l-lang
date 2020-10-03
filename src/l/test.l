enum Option {
    Some(int),
    None
}

// struct Node {
//     val: int,
//     next: Option<Node>,
// }

struct S {
    x: int,
    k: bool,
}


fn main() -> int {
    let s = box box S { x: 5, k: false };
    fib(0)
}

fn fib(n: int) -> int {
    if n < 2 { f() } else { g() }
}

fn f() -> int { 5 }
fn g() -> int { 99 }
