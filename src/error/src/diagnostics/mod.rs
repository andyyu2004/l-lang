mod diagnostic;
mod emitter;

pub use diagnostic::*;
pub use emitter::*;

use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum ErrorFormat {
    Text,
    Json,
}

impl Default for ErrorFormat {
    fn default() -> Self {
        Self::Text
    }
}

impl FromStr for ErrorFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => Err(format!("invalid error format `{}`", s)),
        }
    }
}
