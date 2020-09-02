fn main() -> number {
    /* fn f(n) { */
    /*     if n < 2 { n } else { f(n-1) + f(n-2) } */
    /* }; */
    /* fn () => 5; */
    /* f(5); */
    fib(39)
}

fn fac(n: number) -> number {
    if n < 1 { 1 } else { n * fac(n-1) }
}

fn fib(n: number) -> number {
    if n < 2 { n } else { fib(n-1) + fib(n-2) }
}
