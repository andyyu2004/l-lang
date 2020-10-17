/* enum Option { */
/*     None, */
/*     Some(int), */
/* } */

/* enum NodeOption { */
/*     None, */
/*     Some(&Node), */
/* } */

/* struct Node { */
/*     val: int, */
/*     next: NodeOption, */
/* } */

/* struct S { */
/*     x: int, */
/*     k: bool, */
/* } */

/* enum Either { */
/*     Left(int), */
/*     Right(int), */
/* } */

/* enum Expr { */
/*     Int(int), */
/*     Add(&Expr, &Expr) */
/* } */

extern {
    fn rc<T>(t: &T) -> int;
    fn print(t: int);
}

fn main() -> int {
    /* "string"; */
    /* let expr = box Expr::Add( */
    /*     box Expr::Int(5), */
    /*     box Expr::Int(9), */
    /* ); */
    /* eval(expr); */

    /* return match (1,2,3) { */
    /*     (1,b,3) => b, */
    /*     (a,b,c) => c, */
    /* }; */

    /* return id(5); */
    let boxed = box 5;
    print(*boxed);
    /* print(rc(boxed)); */


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


// fn last(node: &Node) -> int {
//     match node.next {
// 	NodeOption::Some(next) => last(next),
// 	NodeOption::None => node.val,
//     }
// }
//
// fn eval(expr: &Expr) -> int {
//     match *expr {
// 	Expr::Int(i) => i,
// 	Expr::Add(l, r) => eval(l) + eval(r),
//     }
// }
//
//
