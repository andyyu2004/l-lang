extern {
    fn print(i: int);
}


fn main() -> int {
    let f = S::new;
    f(1,false);
    0
}


struct S<T, U> {
    t: T,
    u: U,
}

fn fst(&self)  {
}

impl<T, U> S<T, U> {
    fn new(u: U, t: T) -> Self {
	Self { t, u }
    }

    /* fn fst(&self)  { */
	/* self.t */
    /* } */
}



// impl S<bool, bool> {
//     fn new(t: bool, u: bool) -> Self {
// 	Self { t, u }
//     }
// }
//
// impl S<int, int> {
//     fn new(u: int, t: int) -> Self {
// 	Self { t, u }
//     }
// }
