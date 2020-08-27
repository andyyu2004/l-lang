fn main() -> number {
    fn f(n) {
	if n < 2 { n } else { f(n-1) + f(n-2) }
    };
    fib(40)
}

fn fib(n: number) -> number {
    if n < 2 { n } else { fib(n-1) + fib(n-2) }
}
