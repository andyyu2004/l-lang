
struct S {
    x: int,
    k: bool,
}

enum Option {
    Test
}


fn main() -> int {
    let ptr = double_ptr();
    ***ptr
}

fn double_ptr() -> &&&int {
    box box box 99
}
