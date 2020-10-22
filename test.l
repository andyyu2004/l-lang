// enum Option {
//     None,
//     Some(int),
// }
//
// enum NodeOption {
//     None,
//     Some(&Node),
// }
//
// struct Node {
//     val: int,
//     next: NodeOption,
// }
//
// struct S {
//     x: int,
//     k: bool,
// }
//
// enum Either {
//     Left(int),
//     Right(int),
// }
//
enum Expr {
    Int(int),
    Add(&Expr, &Expr)
}

enum List {
    Next(int, &List),
    Null,
}

extern {
    fn rc<T>(t: &T) -> int;
    fn print(t: int);
    fn addr<T>(t: &T) -> int;
}


fn main() -> int {
    /* let tail = box List::Null; */
    /* let head = box List::Next(4, tail); */

    /* print(addr(tail)); */

    /* let last_ptr = last2(head); */

    /* print(addr(last_ptr)); */

    let expr0 = box Expr::Int(5);
    print(addr(expr0));
    let expr1 = box Expr::Int(9);
    print(addr(expr1));
    let expr = box Expr::Add(expr0, expr1);
    print(addr(expr));
    eval(expr);

    /* let boxed = box 9; */
    /* let cp = boxed; */
    /* rc(cp); */
    /* take_box(boxed); */

    /* let next = box Node { */
    /*     val: 22, */
    /*     next: NodeOption::None, */
    /* }; */
    /* let node = box Node { */
    /*     val: 6, */
    /*     next: NodeOption::Some(next), */
    /* }; */

    /* last(node); */
    8
}


// fn take_box(i: &int) {
//     print(rc(i));
//
// }

fn last2(list: &List) -> &List {
    match list {
	&List::Next(i, l) => {
	    print(addr(l));
	    last2(l)
	},
	&List::Null => list,
    }
}

// fn last(node: &Node) -> int {
//     /* print(rc(node)); */
//     match node.next {
// 	NodeOption::Some(next) => last(next),
// 	NodeOption::None => node.val,
//     }
// }
//
fn eval(expr: &Expr) -> int {
    /* print(rc(expr)); */
    match expr {
	&Expr::Int(i) => i,
	&Expr::Add(l, r) => {
	    print(addr(l));
	    print(addr(r));
	    eval(l) + eval(r)
	}
    }
}


