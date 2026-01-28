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
