use std::fmt::{Display, Formatter, Result};

mod error;
mod metadata;
mod utilities;

pub use error::Error;
pub use metadata::Metadata;

const UNKNOWN: &str = "Unknown";

#[derive(Debug, Eq, PartialEq)]
pub struct Tag {
	pub name: String,
	pub value: String,
}

impl Display for Tag {
	fn fmt(&self, writer: &mut Formatter<'_>) -> Result {
		write!(writer, "{}: {}", self.name, self.value)
	}
}
