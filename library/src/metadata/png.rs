use crate::{
	Error, Tag,
	metadata::{exif, xmp},
	utilities::{
		seek,
		stream::{Be, Bytes},
	},
};
use encoding_rs::WINDOWS_1252;
use flate2::read::ZlibDecoder;
use std::{
	io::{self, ErrorKind, Read, Seek, Write, copy},
	slice,
};

pub fn get<R: Read + Seek>(source: &mut R) -> Result<Vec<Tag>, Error> {
	source.seek_relative(8)?;
	let mut metadata = Vec::new();
	loop {
		let Some(size) = size(source)? else {
			break;
		};
		let mut name = [0; 4];
		source.read_exact(&mut name)?;
		match &name {
			b"eXIf" => {
				let mut data = vec![0; size as usize];
				source.read_exact(&mut data)?;
				exif::get(&data, &mut metadata)?;
				seek!(source, 4)?;
			}
			b"iTXt" => {
				let tag = itxt(source, size)?;
				if tag.name == "XML:com.adobe.xmp" {
					xmp::get(tag.value.as_bytes(), &mut metadata)?;
				} else {
					metadata.push(tag);
				}
				seek!(source, 4)?;
			}
			b"tEXt" => {
				metadata.push(text(source, size)?);
				seek!(source, 4)?;
			}
			b"zTXt" => {
				metadata.push(ztxt(source, size)?);
				seek!(source, 4)?;
			}
			_ => {
				seek!(source, size + 4)?;
			}
		}
	}
	Ok(metadata)
}

pub fn delete<R: Read + Seek, W: Write>(source: &mut R, destination: &mut W) -> Result<(), Error> {
	if copy(&mut source.take(8), destination)? < 8 {
		return Err(Error::File);
	}
	loop {
		let Some(size) = size(source)? else {
			break;
		};
		let mut name = [0; 4];
		source.read_exact(&mut name)?;
		match &name {
			b"IDAT" | b"IEND" | b"IHDR" | b"PLTE" | b"acTL" | b"bKGD" | b"cHRM" | b"cICP"
			| b"fRAc" | b"fcTL" | b"fdAT" | b"gAMA" | b"gIFg" | b"iCCP" | b"sBIT" | b"sRGB"
			| b"sTER" | b"tRNS" => {
				destination.write_all(&size.to_be_bytes())?;
				destination.write_all(&name)?;
				let size = size as u64 + 4;
				if copy(&mut source.take(size), destination)? < size {
					return Err(Error::File);
				}
			}
			_ => {
				seek!(source, size + 4)?;
			}
		}
	}
	Ok(())
}

fn size<R: Read>(source: &mut R) -> Result<Option<u32>, Error> {
	let mut data = [0; 4];
	if let Err(error) = source.read_exact(slice::from_mut(&mut data[0])) {
		if error.kind() == ErrorKind::UnexpectedEof {
			return Ok(None);
		}
		return Err(Error::Io(error));
	};
	source.read_exact(&mut data[1..4])?;
	Ok(Some(u32::from_be_bytes(data)))
}

fn itxt<R: Read>(source: &mut R, size: u32) -> Result<Tag, Error> {
	let mut i = 0;
	let mut name = Vec::new();
	loop {
		let character = Be::u8(source)?;
		i += 1;
		if character == 0 {
			break;
		}
		name.push(character);
	}
	let compression = Be::u8(source)? == 1;
	let zlib = Be::u8(source)? == 0;
	i += 2;
	loop {
		let character = Be::u8(source)?;
		i += 1;
		if character == 0 {
			break;
		}
	}
	loop {
		let character = Be::u8(source)?;
		i += 1;
		if character == 0 {
			break;
		}
	}
	let mut value = vec![0; size as usize - i];
	source.read_exact(&mut value)?;
	let name = String::from_utf8_lossy(&name).to_string();
	let value = if compression {
		if zlib {
			decompress(value.as_slice())
				.map(|value| String::from_utf8_lossy(&value).to_string())
				.unwrap_or("".to_string())
		} else {
			"".to_string()
		}
	} else {
		String::from_utf8_lossy(&value).to_string()
	};
	Ok(Tag { name, value })
}

fn text<R: Read>(source: &mut R, size: u32) -> Result<Tag, Error> {
	let mut name = Vec::new();
	loop {
		let character = Be::u8(source)?;
		if character == 0 {
			break;
		}
		name.push(character);
	}
	let mut value = vec![0; size as usize - name.len() - 1];
	source.read_exact(&mut value)?;
	let name = String::from_utf8_lossy(&name).to_string();
	let (value, _, _) = WINDOWS_1252.decode(&value);
	let value = value.to_string();
	Ok(Tag { name, value })
}

fn ztxt<R: Read>(source: &mut R, size: u32) -> Result<Tag, Error> {
	let mut name = Vec::new();
	loop {
		let character = Be::u8(source)?;
		if character == 0 {
			break;
		}
		name.push(character);
	}
	let compression = Be::u8(source)?;
	let mut data = vec![0; size as usize - name.len() - 1];
	source.read_exact(&mut data)?;
	let mut value = "".to_string();
	if compression == 0 {
		if let Ok(data) = decompress(data.as_slice()) {
			let (data, _, _) = WINDOWS_1252.decode(&data);
			value = data.to_string()
		}
	}
	let name = String::from_utf8_lossy(&name).to_string();
	Ok(Tag { name, value })
}

fn decompress(data: &[u8]) -> Result<Vec<u8>, io::Error> {
	let mut decoder = ZlibDecoder::new(data);
	let mut data = Vec::new();
	decoder.read_to_end(&mut data)?;
	Ok(data)
}
