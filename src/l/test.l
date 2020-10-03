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
    let opt = Option::None;
    let opt = Option::Some(5);
    match opt {
	Option::Some(_) => 99,
	Option::None => 77,
    }
}


