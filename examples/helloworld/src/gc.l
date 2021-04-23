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

enum Expr {
    Int(int),
    Add(&Expr, &Expr),
}

enum List {
    Next(int, &List),
    Null,
}

extern "l-intrinsic" {
    fn print(t: int);
    fn addr<T>(t: &T) -> int;
}

fn run() -> int {
    let tail = box List::Null;
    let head = box List::Next(4, tail);

    print(addr(tail));

    let last_ptr = last2(head);

    print(addr(last_ptr));

    let expr0 = box Expr::Int(5);
    print(addr(expr0));
    let expr1 = box Expr::Int(9);
    print(addr(expr1));
    let expr = box Expr::Add(expr0, expr1);
    print(addr(expr));
    print(eval(expr));

    let boxed = box 9;
    let cp = boxed;

    let next = box Node { val: 22, next: NodeOption::None };
    let node = box Node { val: 6, next: NodeOption::Some(next) };

    print(last(node));
    8
}

fn last2(list: &List) -> &List {
    match list {
        &List::Null => list,
        &List::Next(i, l) => {
            print(addr(l));
            last2(l)
        }
    }
}

fn last(node: &Node) -> int {
    match node.next {
        NodeOption::Some(next) => last(next),
        NodeOption::None => node.val,
    }
}

fn eval(expr: &Expr) -> int {
    match expr {
        &Expr::Int(i) => i,
        &Expr::Add(l, r) => {
            print(addr(l));
            print(addr(r));
            eval(l) + eval(r)
        }
    }
}
