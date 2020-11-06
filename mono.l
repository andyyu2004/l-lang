
/* fn f(s: S) {} */

struct S<T> {
    t: T
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
