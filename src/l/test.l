enum Option {
    None,
    Some(int),
}

enum NodeOption {
    None,
    Some(&Node),
}

struct Node {
    val: int,
    next: NodeOption,
}

struct S {
    x: int,
    k: bool,
}

enum Either {
    Left(int),
    Right(int),
}

// enum Expr {
//     Int(int),
//     Add(&Expr, &Expr)
// }


fn main() -> int {
    let node = Node {
	val: 9,
	next: NodeOption::None,
    };

    let head = Node {
	val: 4,
	next: NodeOption::Some(box node),
    };

    match head.next {
	NodeOption::Some(n) => n.val,
	NodeOption::None => 0,
    }
}


