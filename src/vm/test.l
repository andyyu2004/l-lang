
/* fn main() -> int { */
/*     let ptr = box 5; */
/*     hold_on(ptr); */
/*     89 */
/* } */

fn main() -> int { return fib(10) }

fn fib(n: int) -> int {
    // note one branch is explicit return and one is not
    return if n < 2 { n } else { return fib(n - 1) + fib(n - 2) }
}

fn hold_on(ptr: &int) {
    let copy = ptr;
}
