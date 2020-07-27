use crate::error::Diagnostics;
use std::ops::Deref;

#[derive(Default)]
crate struct Session {
    diagnostics: Diagnostics,
}

impl Deref for Session {
    type Target = Diagnostics;

    fn deref(&self) -> &Self::Target {
        &self.diagnostics
    }
}
