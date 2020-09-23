
 struct Node {
     x: int,
     next: Node,
 }

// struct S {
//     x: int,
//     y: bool,
// }

enum Option {
    Some(int),
    None,
}

impl Option {
    fn unwrap() -> int {
	99
    }
}



fn main() -> int {
    let x = box 5;
    return *x
}

