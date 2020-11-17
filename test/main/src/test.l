extern {
    fn print(i: int);
}

fn uninit() {
    let x = 5;
    print(x);
}

