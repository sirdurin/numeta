use crate::{Error, Tag, metadata::xmp};
use lopdf::{Document, Object, Reader, Stream};
use std::{
	collections::BTreeMap,
	io::{Read, Seek, Write},
};

macro_rules! read {
	($source:expr, $function:expr) => {{
		let mut data = Vec::new();
		$source.read_to_end(&mut data)?;
		let document = Reader {
			buffer: &data,
			document: Document::new(),
			encryption_state: None,
			password: None,
			raw_objects: BTreeMap::new(),
		}
		.read($function)?;
		if document.is_encrypted() && document.encryption_state.is_none() {
			Err(Error::Encryption)
		} else {
			Ok(document)
		}
	}};
}

pub fn get<R: Read + Seek>(source: &mut R) -> Result<Vec<Tag>, Error> {
	let mut metadata = Vec::new();
	let document = read!(source, None)?;
	if let Ok(value) = document
		.trailer
		.get(b"DocChecksum")
		.and_then(Object::as_name)
	{
		let name = "DocChecksum".to_string();
		let value = String::from_utf8_lossy(value).to_string();
		metadata.push(Tag { name, value });
	}
	if let Ok(dictionary) = document
		.trailer
		.get(b"Info")
		.and_then(Object::as_reference)
		.and_then(|reference| document.get_object(reference))
		.and_then(Object::as_dict)
	{
		for (name, value) in dictionary.iter() {
			let name = String::from_utf8_lossy(name).to_string();
			let value = match value {
				Object::Boolean(value) => value.to_string(),
				Object::Integer(value) => value.to_string(),
				Object::Name(value) => String::from_utf8_lossy(value).to_string(),
				Object::Real(value) => value.to_string(),
				Object::String(value, _) => String::from_utf8_lossy(value).to_string(),
				_ => "".to_string(),
			};
			metadata.push(Tag { name, value });
		}
	};
	if let Ok(data) = document
		.catalog()
		.and_then(|catalog| catalog.get(b"Metadata"))
		.and_then(Object::as_reference)
		.and_then(|reference| document.get_object(reference))
		.and_then(Object::as_stream)
		.and_then(Stream::get_plain_content)
	{
		let _ = xmp::get(&data, &mut metadata);
	}
	Ok(metadata)
}

pub fn delete<R: Read, W: Write>(source: &mut R, destination: &mut W) -> Result<(), Error> {
	let mut document = read!(
		source,
		Some(|number, object| {
			match object.type_name() {
				Ok(b"DocTimeStamp") | Ok(b"Metadata") | Ok(b"Sig") => None,
				_ => {
					if let Ok(dictionary) = object.as_dict_mut() {
						dictionary.remove(b"LastModified");
						dictionary.remove(b"Metadata");
						dictionary.remove(b"PieceInfo");
					} else if let Ok(stream) = object.as_stream_mut() {
						stream.dict.remove(b"LastModified");
						stream.dict.remove(b"Metadata");
						stream.dict.remove(b"PieceInfo");
					}
					Some((number, object.clone()))
				}
			}
		})
	)?;
	document.trailer.remove(b"DocChecksum");
	document.trailer.remove(b"Info");
	if let Ok(catalog) = document.catalog_mut() {
		catalog.remove(b"Lang");
		catalog.remove(b"Legal");
		catalog.remove(b"Perms");
		catalog.remove(b"SpiderInfo");
	}
	document.prune_objects();
	document.renumber_objects();
	document.save_to(destination)?;
	Ok(())
}
