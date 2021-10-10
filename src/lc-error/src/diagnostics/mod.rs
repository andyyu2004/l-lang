mod diagnostic;
mod emitter;

pub use diagnostic::*;
pub use emitter::*;

use serde::Deserialize;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum ErrorFormat {
    Text,
    Json,
}

impl Display for ErrorFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorFormat::Text => write!(f, "text"),
            ErrorFormat::Json => write!(f, "json"),
        }
    }
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
            _ => Err(format!("invalid error format `{}` (available options are [text, json])", s)),
        }
    }
}
