fn main() -> number {
    fib(35)
}

fn fib(n: number) -> number {
    if n < 2 { n } else { fib(n-1) + fib(n-2) }
}
