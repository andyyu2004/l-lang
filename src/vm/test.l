
struct S {
    x: int,
    y: bool,
}

fn main() -> int {
    let s = box S { x: 5, y: false };
    let x = box 5;
    y;
    mutate(x);
    *x
    /* (*s).x */
    /* (1, false).1 */
}

fn mutate(ptr: &mut int) {
    *ptr = 9;
}

