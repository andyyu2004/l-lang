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
    fib(5)
}

fn fib(n: int) -> int {
    if n < 2 { 1 } else { n * fib(n-1) }
}
