use crate::{Error, Tag};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};

pub mod exif;
pub mod iptc;
pub mod jpeg;
pub mod png;
#[cfg(test)]
mod test;
pub mod webp;
pub mod xmp;

#[derive(Debug)]
pub enum Metadata {
	Jpeg,
	Png,
	Webp,
}

impl Metadata {
	pub fn guess<R: Read + Seek>(
		source: &mut R,
		_: Option<&str>,
	) -> Result<Option<Metadata>, Error> {
		let mut data = [0; 8];
		if let Err(error) = source.read_exact(&mut data) {
			if error.kind() == ErrorKind::UnexpectedEof {
				return Ok(None);
			}
			return Err(Error::Io(error));
		};
		source.seek(SeekFrom::Start(0))?;
		match data[0] {
			b'R' => {
				if &data[1..4] == b"IFF" {
					return Ok(Some(Metadata::Webp));
				}
			}
			0x89 => {
				if data[1..8] == [0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
					return Ok(Some(Metadata::Png));
				}
			}
			0xFF => {
				if data[1..3] == [0xD8, 0xFF] {
					return Ok(Some(Metadata::Jpeg));
				}
			}
			_ => {}
		}
		Ok(None)
	}

	pub fn get<R: Read + Seek>(&self, source: &mut R) -> Result<Vec<Tag>, Error> {
		match self {
			Metadata::Jpeg => jpeg::get(source),
			Metadata::Png => png::get(source),
			Metadata::Webp => webp::get(source),
		}
	}

	pub fn delete<R: Read + Seek, W: Write>(
		&self,
		source: &mut R,
		destination: &mut W,
	) -> Result<(), Error> {
		match self {
			Metadata::Jpeg => jpeg::delete(source, destination),
			Metadata::Png => png::delete(source, destination),
			Metadata::Webp => webp::delete(source, destination),
		}
	}
}
