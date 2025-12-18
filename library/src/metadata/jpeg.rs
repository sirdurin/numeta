use crate::{
	Error, Tag,
	metadata::{exif, iptc, xmp},
	utilities::{
		seek,
		stream::{Be, Bytes},
	},
};
use std::{
	io::{ErrorKind, Read, Seek, SeekFrom, Write, copy},
	slice,
};

pub fn get<R: Read + Seek>(source: &mut R) -> Result<Vec<Tag>, Error> {
	let mut metadata = Vec::new();
	loop {
		let Some(marker) = marker(source)? else {
			break;
		};
		if marker[0] != 0xFF {
			return Err(Error::File);
		}
		match marker[1] {
			0xC0..=0xCF | 0xDB..=0xDF => {
				let size = Be::u16(source)? - 2;
				if size > 0 {
					seek!(source, size)?;
				}
			}
			0xD0..=0xD7 | 0xDA => loop {
				if Be::u8(source)? == 0xFF {
					if Be::u8(source)? > 0 {
						source.seek(SeekFrom::Current(-2))?;
						break;
					}
				}
			},
			0xD8..=0xD9 => {}
			0xE0 => {
				let size = Be::u16(source)? - 2;
				if size < 14 {
					seek!(source, size)?;
					continue;
				}
				let mut name = [0; 5];
				source.read_exact(&mut name)?;
				if &name == b"JFIF\0" {
					seek!(source, 7)?;
					let width = Be::u8(source)?;
					let height = Be::u8(source)?;
					if width > 0 && height > 0 {
						metadata.push(Tag {
							name: "Thumbnail".to_string(),
							value: "<data>".to_string(),
						});
						seek!(source, 3 * width * height)?;
					}
				} else {
					seek!(source, size - 5)?;
				}
			}
			0xE1 => {
				let size = Be::u16(source)? - 2;
				let mut data = vec![0; size as usize];
				source.read_exact(&mut data)?;
				if size > 29 && &data[0..29] == b"http://ns.adobe.com/xap/1.0/\0" {
					xmp::get(&data[29..], &mut metadata)?;
				} else if size > 6 && &data[0..6] == b"Exif\0\0" {
					exif::get(&data[6..], &mut metadata)?;
				}
			}
			0xED => {
				let size = Be::u16(source)? - 2;
				let mut data = vec![0; size as usize];
				source.read_exact(&mut data)?;
				if size > 14 && &data[0..14] == b"Photoshop 3.0\0" {
					iptc::get(&data[14..], &mut metadata)?;
				}
			}
			_ => {
				let size = Be::u16(source)? - 2;
				seek!(source, size)?;
			}
		}
	}
	Ok(metadata)
}

pub fn delete<R: Read + Seek, W: Write>(source: &mut R, destination: &mut W) -> Result<(), Error> {
	loop {
		let Some(marker) = marker(source)? else {
			break;
		};
		if marker[0] != 0xFF {
			return Err(Error::File);
		}
		match marker[1] {
			0xC0..=0xCF | 0xDB..=0xDF => {
				destination.write_all(&marker)?;
				let size = Be::u16(source)?;
				destination.write_all(&size.to_be_bytes())?;
				if size > 2 {
					let size = size as u64 - 2;
					if copy(&mut source.take(size), destination)? < size {
						return Err(Error::File);
					}
				}
			}
			0xD0..=0xD7 => {
				destination.write_all(&marker)?;
				data(source, destination)?;
			}
			0xD8..=0xD9 => {
				destination.write_all(&marker)?;
			}
			0xDA => {
				destination.write_all(&marker)?;
				let size = Be::u16(source)?;
				destination.write_all(&size.to_be_bytes())?;
				if size > 2 {
					let size = size as u64 - 2;
					if copy(&mut source.take(size), destination)? < size {
						return Err(Error::File);
					}
				}
				data(source, destination)?;
			}
			_ => {
				let size = Be::u16(source)? - 2;
				seek!(source, size)?;
			}
		}
	}
	Ok(())
}

fn marker<R: Read>(source: &mut R) -> Result<Option<[u8; 2]>, Error> {
	let mut code = [0; 2];
	if let Err(error) = source.read_exact(slice::from_mut(&mut code[0])) {
		if error.kind() == ErrorKind::UnexpectedEof {
			return Ok(None);
		}
		return Err(Error::Io(error));
	};
	source.read_exact(slice::from_mut(&mut code[1]))?;
	Ok(Some(code))
}

fn data<R: Read + Seek, W: Write>(source: &mut R, destination: &mut W) -> Result<(), Error> {
	loop {
		let value = Be::u8(source)?;
		if value == 0xFF {
			let next = Be::u8(source)?;
			if next == 0 {
				destination.write_all(&[value, next])?;
			} else {
				source.seek(SeekFrom::Current(-2))?;
				break;
			}
		} else {
			destination.write_all(&[value])?;
		}
	}
	Ok(())
}
