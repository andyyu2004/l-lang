struct S<T, U> {
    t: T,
    u: U,
}

extern "l-intrinsic" {
    fn print(i: int);
}

pub fn run() {
    let s = S { t: 4, u: false };
    let s1 = S::new(false, 8);
    print(s.t);
    print(s1.u);
}

// impl<T, U> S<T, U> {
//     fn new(u: U, t: T) -> Self {
//         Self { t, u }
//     }

//     fn fst(self) -> T {
//         self.t
//     }
// }



impl S<bool, int> {
    fn new(t: bool, u: int) -> Self {
        Self { t, u }
    }
}

// impl S<int, int> {
//     fn new(u: int, t: int) -> Self {
// 	Self { t, u }
//     }
// }
