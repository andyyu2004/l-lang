fn main() -> int {
    let (x, x) = (1,2); //~ ERROR identifier `x` bound more than once in the same pattern
    0
}
