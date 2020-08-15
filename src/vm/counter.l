fn main() -> number {
    let counter = mk_counter();
    counter();
    counter();
    counter();
    counter()
}

fn mk_counter() -> fn() -> number {
    let mut i = 0;
    fn() => i = i + 1
}
