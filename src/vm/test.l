fn main() {
    f(5, false);
    /* f(false, false); */
}


fn f<T>(t: T, u: T) -> T {
    t
}


