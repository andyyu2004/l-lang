
// struct Node {
//     x: int,
//     next: Node,
// }

// struct S {
//     x: int,
//     y: bool,
// }

enum Option {
    Some(int),
    None(),
}


fn main() -> int {
    let opt = Option::None();
    match opt {
        Option::Some(x) => x,
        Option::None => 99,
    }
}



