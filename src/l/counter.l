fn main() -> int {
    let x: bool = 5;
    let counter = mk_counter();
    counter();
    counter();
    counter();
    counter()
}

fn mk_counter() -> fn() -> int {
    let mut i = 0;
    fn() => i = i + 1
}
