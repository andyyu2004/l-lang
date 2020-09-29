
struct S {
    x: int,
    k: bool,
}

fn main() -> int {
    let boxed = box S { x: 5, k: false };
    8
}

