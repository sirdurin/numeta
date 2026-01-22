use std::{
	error,
	fmt::{Debug, Display, Formatter, Result},
	io::{self, ErrorKind},
	num::ParseIntError,
	str::Utf8Error,
};

pub enum Error {
	Encryption,
	File,
	Io(io::Error),
	Metadata,
}

impl Debug for Error {
	fn fmt(&self, writer: &mut Formatter<'_>) -> Result {
		write!(writer, "{}", self)
	}
}

impl Display for Error {
	fn fmt(&self, writer: &mut Formatter<'_>) -> Result {
		match self {
			Error::Encryption => write!(writer, "encrypted file"),
			Error::File => write!(writer, "invalid file"),
			Error::Io(error) => write!(writer, "{}", error),
			Error::Metadata => write!(writer, "invalid metadata"),
		}
	}
}

impl error::Error for Error {
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		match self {
			Error::Io(error) => Some(error),
			_ => None,
		}
	}
}

impl From<io::Error> for Error {
	fn from(error: io::Error) -> Self {
		if error.kind() == ErrorKind::UnexpectedEof {
			Error::File
		} else {
			Error::Io(error)
		}
	}
}

impl From<lopdf::Error> for Error {
	fn from(_: lopdf::Error) -> Self {
		Error::File
	}
}

impl From<ParseIntError> for Error {
	fn from(_: ParseIntError) -> Self {
		Error::Metadata
	}
}

impl From<quick_xml::Error> for Error {
	fn from(_: quick_xml::Error) -> Self {
		Error::Metadata
	}
}

impl From<Utf8Error> for Error {
	fn from(_: Utf8Error) -> Self {
		Error::Metadata
	}
}
