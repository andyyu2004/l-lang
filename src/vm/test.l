
struct S {
    x: int,
    k: bool,
}


fn main() -> int {
    let ptr = f2();
    /* **ptr */
    5
}

fn f2() -> &&int {
    box f3()
}

fn f3() -> &int {
    box 99
}
