extern {
    fn print(i: int);
}

struct S {
    x: int,
}

fn uninit() {
    let mut s = S { x: 5 };
    s.x = 10;
    let mut x = 5;
    x = 10;
    print(x);
}

