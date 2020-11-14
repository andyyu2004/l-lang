use crate::Module;
use std::fmt::{self, Display, Formatter};

/// top level ast representation that stores entire contents of the program being compiled
#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
    /// implicit top level module
    pub module: Module,
}

impl Display for Ast {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.module)
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            writeln!(f, "{}", item)?;
        }
        Ok(())
    }
}
