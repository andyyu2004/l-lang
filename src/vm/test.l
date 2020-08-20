
struct S {
    x: number,
}

fn main() -> number {
    // S { x: 5 }
    f();
    (fn () => 5)()
}

fn f() -> S {
    S { x: 5 }
}

// fn fib(n: number) -> number {
//    return if n < 2 { return n; } else { return fib(n - 1) + fib(n - 2); };
//}






