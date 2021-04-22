extern {
    fn print(i: int);
}

/* fn main() -> int { */
    /* print(fib(40)); */
    /* return 0; */
/* } */


fn fib(n: int) -> int {
    if n < 2 { n } else { fib(n-1) + fib(n-2) }
}


