extern {
    fn print(i: int);
}

fn uninit() {
    let x;
    print(x);
}

