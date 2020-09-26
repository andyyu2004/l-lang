
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

enum Option<T> {
    Some(T),
    None,
}

struct G<T> {
    t: T
}


// impl Option {
//     fn unwrap() -> int {
// 	99
//     }
// }


fn id<T>(x: T) -> T {
    x
}

fn main() -> int {
    let g = G { t: 5 };
    g.t
}

