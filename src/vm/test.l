
struct Node {
    x: int,
    next: Node,
}

struct S {
    x: int,
    y: bool,
}

enum K {
    T,
    F,
}


fn main() -> int {
    let s = box S { x: 5, y: false };
    let node = Node { x: 5, next: node };
    5
}


fn mutate(ptr: &mut int) {
    *ptr = 9;
}

