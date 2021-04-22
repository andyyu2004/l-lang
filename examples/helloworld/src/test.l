extern {
    fn print(i: int);
}

struct S {
    x: int,
}

fn uninit() {
    let x;
    let y  = x;
    print(x);
    let mut s = S { x: 5 };
    s.x = 9;
    let s = box S { x : 4 };
    let mut x = 5;
    x = 10;
    print(s.x)
}



