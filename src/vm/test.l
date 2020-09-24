
 struct Node {
     x: int,
     next: Node,
 }

// struct S {
//     val: int,
//     next: OptionS,
// }
//
// enum OptionS {
//     Some(S),
//     None,
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

