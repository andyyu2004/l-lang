
/* fn f(s: S) {} */

struct S<T, U> {
    t: T,
    u: U,
}

struct K<T, U> {
    s: S<T, U>
}

enum Option<T> {
    Some(T),
    None,
}


fn main() -> int {
    0
}
