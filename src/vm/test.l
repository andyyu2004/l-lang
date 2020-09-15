
struct S {
    x: int,
    y: bool,
}


fn main() -> int {
    let s = box S { x: 5, y: false };
    let ref = unsafe { &5 };
    mutate(ref);
    *ref
}

fn mutate(ptr: &mut int) {
    *ptr = 9;
}

