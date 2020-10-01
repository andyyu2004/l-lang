enum Option {
    Some(int),
    None
}

// struct Node {
//     val: int,
//     next: Option<Node>,
// }

struct S {
    x: int,
    k: bool,
}


fn main() -> int {
    let x = Option::None;
    let i = 5;
    match i {
	5 => 99,
	_ => 69,
    }
}

