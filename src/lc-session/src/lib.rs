#[macro_use]
extern crate log;

mod config;
mod profiling;

pub use config::CompilerOptions;

use lc_error::Diagnostics;
use profiling::Profiler;
use std::ops::Deref;

pub struct Session {
    pub prof: Profiler,
    pub opts: CompilerOptions,
    diagnostics: Diagnostics,
}

impl Session {
    pub fn create(opts: CompilerOptions) -> Self {
        Self {
            diagnostics: Diagnostics::with_error_format(opts.error_format),
            opts,
            prof: Default::default(),
        }
    }
}

impl Deref for Session {
    type Target = Diagnostics;

    fn deref(&self) -> &Self::Target {
        &self.diagnostics
    }
}
