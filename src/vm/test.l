
struct S {
    x: number,
}

fn main() -> number { return fib(10); }

fn fib(n: number) -> number {
    return if n < 2 { return n; } else { return fib(n - 1) + fib(n - 2); };
}






