
/* struct Node { */
/*     x: int, */
/*     next: Node, */
/* } */

/* struct S { */
/*     x: int, */
/*     y: bool, */
/* } */


fn main() -> int {
    /* let s = box S { x: 5, y: false }; */
    /* let ref = unsafe { &5 }; */
    fn f() { f };
    /* mutate(ref); */
    /* *ref */
    5
}


fn mutate(ptr: &mut int) {
    *ptr = 9;
}

