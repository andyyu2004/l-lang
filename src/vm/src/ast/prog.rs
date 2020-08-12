use super::{Expr, Item, P};
use std::fmt::{self, Display, Formatter};

/// top level ast representation that stores entire contents of the program being compiled
#[derive(Debug)]
pub struct Prog {
    pub items: Vec<P<Item>>,
}

impl Display for Prog {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            writeln!(f, "{}", item)?;
        }
        Ok(())
    }
}
