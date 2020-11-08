use super::{Item, P};
use std::fmt::{self, Display, Formatter};

/// top level ast representation that stores entire contents of the program being compiled
#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
    pub items: Vec<P<Item>>,
}

impl Display for Ast {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            writeln!(f, "{}", item)?;
        }
        Ok(())
    }
}
