
// struct Node {
//     x: int,
//     next: Node,
// }

// struct S {
//     x: int,
//     y: bool,
// }

enum Option {
    Some { x: int },
    None,
}

enum Option2 {
    Some(int),
    None,
}


fn main() -> int {
    let none = Option::None;
    let some = Option::Some { x: 9 };
    let some2 = Option2::Some(5);
    9
}



