use prelude::print;

extern {
    fn print(i: int);
}


fn main() -> int {
    let f = S::new;
    f(1,false);
    0
}

type P<T> = S<T, int>;

fn construct_p() -> P<bool> {
    P { t: false, u: 9 }
}

struct S<T, U> {
    t: T,
    u: U,
}


impl<T, U> S<T, U> {
    fn new(u: U, t: T) -> Self {
	Self { t, u }
    }

    fn fst(self) -> T {
	self.t
	/* self.fst() */
    }
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
