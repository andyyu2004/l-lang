fn main() {
    let f = fn(x) => false;
    let g = fn(p) => p(3);
    g(f);
}
