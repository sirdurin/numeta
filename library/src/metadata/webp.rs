use crate::{
	Error, Tag,
	metadata::{exif, xmp},
	utilities::{
		seek,
		stream::{Bytes, Le},
	},
};
use std::{
	io::{ErrorKind, Read, Seek, Write, copy},
	slice,
};

pub fn get<R: Read + Seek>(source: &mut R) -> Result<Vec<Tag>, Error> {
	source.seek_relative(12)?;
	let mut metadata = Vec::new();
	loop {
		let Some(code) = code(source)? else {
			break;
		};
		let size = Le::u32(source)?;
		match &code {
			b"EXIF" => {
				let size = if size % 2 > 0 { size + 1 } else { size };
				let mut data = vec![0; size as usize];
				source.read_exact(&mut data)?;
				exif::get(&data, &mut metadata)?;
			}
			b"XMP " => {
				let mut data = vec![0; size as usize];
				source.read_exact(&mut data)?;
				xmp::get(&data, &mut metadata)?;
				if size % 2 > 0 {
					seek!(source, 1)?;
				}
			}
			_ => {
				let size = if size % 2 > 0 { size + 1 } else { size };
				seek!(source, size)?;
			}
		}
	}
	Ok(metadata)
}

pub fn delete<R: Read + Seek, W: Write>(source: &mut R, destination: &mut W) -> Result<(), Error> {
	source.seek_relative(12)?;
	let mut data = Vec::new();
	loop {
		let Some(code) = code(source)? else {
			break;
		};
		let size = Le::u32(source)?;
		match &code {
			b"ALPH" | b"ANIM" | b"ANMF" | b"ICCP" | b"VP8 " | b"VP8L" | b"VP8X" => {
				data.extend_from_slice(&code);
				data.extend_from_slice(&size.to_le_bytes());
				let size = if size % 2 > 0 { size + 1 } else { size } as u64;
				if copy(&mut source.take(size), &mut data)? < size {
					return Err(Error::File);
				}
			}
			_ => {
				let size = if size % 2 > 0 { size + 1 } else { size };
				seek!(source, size)?;
			}
		}
	}
	let size = data.len() as u32 + 4;
	destination.write_all(b"RIFF")?;
	destination.write_all(&size.to_le_bytes())?;
	destination.write_all(b"WEBP")?;
	destination.write_all(&data)?;
	Ok(())
}

fn code<R: Read>(source: &mut R) -> Result<Option<[u8; 4]>, Error> {
	let mut code = [0; 4];
	if let Err(error) = source.read_exact(slice::from_mut(&mut code[0])) {
		if error.kind() == ErrorKind::UnexpectedEof {
			return Ok(None);
		}
		return Err(Error::Io(error));
	};
	source.read_exact(&mut code[1..4])?;
	Ok(Some(code))
}
