
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
    let opt = Option::None;
    match 5 {
	x => 5,
	y => 6,
    }
}



