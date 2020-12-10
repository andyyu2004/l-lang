fn main() -> int {
    let p = false;
    let q = true;
    match (p, q) {
        (false, false) => false,
        _ => true,
        (true, true) => true, //~ WARNING redundant pattern

    };
    0
}
