fn main() {
    let p = false;
    let q = true;
    match (p, q) {
        (false, false) => false, //~ ERROR non-exhaustive match
    }
}
