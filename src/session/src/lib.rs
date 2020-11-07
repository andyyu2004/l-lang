#[macro_use]
extern crate log;

mod profiling;

use error::Diagnostics;
use profiling::Profiler;
use std::ops::Deref;

#[derive(Default)]
pub struct Session {
    pub prof: Profiler,
    diagnostics: Diagnostics,
}

impl Deref for Session {
    type Target = Diagnostics;

    fn deref(&self) -> &Self::Target {
        &self.diagnostics
    }
}
