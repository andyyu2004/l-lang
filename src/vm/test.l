
struct S {
    x: int,
    k: bool,
}


fn main() -> int {
    let ptr = f2();
    **ptr
}

fn f1() -> &&&int {
    box f2()
}

fn f2() -> &&int {
    box f3()
}

fn f3() -> &int {
    box 99
}
