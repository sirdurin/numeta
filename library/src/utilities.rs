macro_rules! min {
	($a:expr, $b:expr) => {
		if $a < $b { $a } else { $b }
	};
}

macro_rules! seek {
	($source:expr, $size:expr) => {
		$source.seek_relative($size.into())
	};
}

pub(crate) use min;
pub(crate) use seek;

pub mod bytes {
	use std::mem;

	pub trait Bytes {
		fn i32(source: &[u8]) -> i32;
		fn u8(source: &[u8]) -> u8;
		fn u16(source: &[u8]) -> u16;
		fn u32(source: &[u8]) -> u32;
	}

	pub struct Be;
	pub struct Le;

	macro_rules! create_function {
		($data:ident, $function:ident) => {
			fn $data(source: &[u8]) -> $data {
				let mut value = [0; mem::size_of::<$data>()];
				value.copy_from_slice(&source[0..mem::size_of::<$data>()]);
				$data::$function(value)
			}
		};
	}

	impl Bytes for Be {
		create_function!(i32, from_be_bytes);
		create_function!(u8, from_be_bytes);
		create_function!(u16, from_be_bytes);
		create_function!(u32, from_be_bytes);
	}

	impl Bytes for Le {
		create_function!(i32, from_le_bytes);
		create_function!(u8, from_le_bytes);
		create_function!(u16, from_le_bytes);
		create_function!(u32, from_le_bytes);
	}
}

pub mod stream {
	use crate::Error;
	use std::{io::Read, mem};

	pub trait Bytes {
		fn u8<R: Read>(source: &mut R) -> Result<u8, Error>;
		fn u16<R: Read>(source: &mut R) -> Result<u16, Error>;
		fn u32<R: Read>(source: &mut R) -> Result<u32, Error>;
	}

	pub struct Be;
	pub struct Le;

	macro_rules! create_function {
		($data:ident, $function:ident) => {
			fn $data<R: Read>(source: &mut R) -> Result<$data, Error> {
				let mut value = [0; mem::size_of::<$data>()];
				source.read_exact(&mut value)?;
				Ok($data::$function(value))
			}
		};
	}

	impl Bytes for Be {
		create_function!(u8, from_be_bytes);
		create_function!(u16, from_be_bytes);
		create_function!(u32, from_be_bytes);
	}

	impl Bytes for Le {
		create_function!(u8, from_le_bytes);
		create_function!(u16, from_le_bytes);
		create_function!(u32, from_le_bytes);
	}
}

pub mod xml {
	use crate::Error;
	use quick_xml::{Reader, events::Event};
	use std::io::BufRead;

	pub fn parse_name(name: &[u8]) -> (String, String) {
		let name = String::from_utf8_lossy(name).to_string();
		let mut tokens = name.splitn(2, ':');
		if let Some(value_1) = tokens.nth(0) {
			if let Some(value_2) = tokens.nth(0) {
				let name = value_2.to_string();
				let namespace = value_1.to_string();
				(name, namespace)
			} else {
				let name = value_1.to_string();
				let namespace = "".to_string();
				(name, namespace)
			}
		} else {
			let name = "".to_string();
			let namespace = "".to_string();
			(name, namespace)
		}
	}

	pub fn skip<R: BufRead>(source: &mut Reader<R>, name: &[u8]) -> Result<(), Error> {
		let mut data = Vec::new();
		loop {
			match source.read_event_into(&mut data)? {
				Event::End(end) if end.name().as_ref() == name => break,
				Event::End(_) | Event::Eof => return Err(Error::Metadata),
				Event::Start(start) => skip(source, start.name().as_ref())?,
				_ => {}
			}
		}
		Ok(())
	}

	pub fn parse_string<R: BufRead>(source: &mut Reader<R>, name: &[u8]) -> Result<String, Error> {
		let mut data = Vec::new();
		let mut string = None;
		loop {
			match source.read_event_into(&mut data)? {
				Event::End(end) if end.name().as_ref() == name => break,
				Event::End(_) | Event::Eof => return Err(Error::Metadata),
				Event::Start(_) => {
					// TODO
				}
				Event::Text(value) => {
					string = Some(String::from_utf8_lossy(&value).trim_ascii().to_string());
				}
				_ => {}
			}
		}
		Ok(string.unwrap_or("".to_string()))
	}
}
