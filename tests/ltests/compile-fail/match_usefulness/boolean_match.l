fn main() -> int {
    let p = false;
    let q = true;
    match (p, q) { //~ ERROR non-exhaustive match expression
        (false, false) => false,
    };
    0
}
