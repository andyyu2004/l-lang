use super::{Expr, Item, P};

/// top level ast representation that stores entire contents of the program being compiled
#[derive(Debug)]
crate struct Prog {
    pub items: Vec<P<Item>>,
}

/// wraps a expression in a implicit main function to form a program
impl From<Expr> for Prog {
    fn from(expr: Expr) -> Self {
        let f = Item::from(expr);
        Self { items: vec![box f] }
    }
}
