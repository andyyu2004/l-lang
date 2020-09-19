
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
    None,
}


fn main() -> int {
    let opt = Option::Some(5);
    let (x, x) = (1,2);
    let f = fn(k) => k;
    let g = fn(k) => 1 + k;
    return f(g(9));

    // match opt {
    //     Option::Some(x) => 1000,
    //     Option::None => 99,
    // }
}



