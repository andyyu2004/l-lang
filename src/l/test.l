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
    let opt = Option::Some(9);
    match opt {
	Option::Some(x) => x,
	Option::None => 77,
    }
}


