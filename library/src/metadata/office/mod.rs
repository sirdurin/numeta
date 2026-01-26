use crate::{
	Error, Tag, UNKNOWN,
	utilities::xml::{parse_name, parse_string, skip},
};
use quick_xml::{events::Event, name::QName, reader::Reader, writer::Writer};
use std::io::{BufReader, Read, Seek, Write};
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter, write::SimpleFileOptions};

#[cfg(test)]
mod test;

pub fn get<R: Read + Seek>(source: &mut R) -> Result<Vec<Tag>, Error> {
	let mut metadata = Vec::new();
	let mut archive = ZipArchive::new(source)?;
	for i in 0..archive.len() {
		let entry = archive.by_index(i)?;
		match entry.name() {
			"docProps/app.xml" => parse_application(BufReader::new(entry), &mut metadata)?,
			"docProps/core.xml" => parse_core(BufReader::new(entry), &mut metadata)?,
			_ => {}
		}
	}
	Ok(metadata)
}

fn parse_application<R: Read>(source: BufReader<R>, metadata: &mut Vec<Tag>) -> Result<(), Error> {
	let mut data = Vec::new();
	let mut source = Reader::from_reader(source);
	loop {
		match source.read_event_into(&mut data)? {
			Event::Eof => break,
			Event::Start(start) => {
				let (name, _) = parse_name(start.name().as_ref());
				if name == "Properties" {
					parse_properties(&mut source, start.name(), metadata)?;
				}
			}
			_ => {}
		}
	}
	Ok(())
}

fn parse_core<R: Read>(source: BufReader<R>, metadata: &mut Vec<Tag>) -> Result<(), Error> {
	let mut data = Vec::new();
	let mut source = Reader::from_reader(source);
	loop {
		match source.read_event_into(&mut data)? {
			Event::Eof => break,
			Event::Start(start) => {
				let (name, _) = parse_name(start.name().as_ref());
				if name == "coreProperties" {
					parse_core_properties(&mut source, start.name(), metadata)?;
				}
			}
			_ => {}
		}
	}
	Ok(())
}

fn parse_core_properties<R: Read>(
	source: &mut Reader<BufReader<R>>,
	name: QName,
	metadata: &mut Vec<Tag>,
) -> Result<(), Error> {
	loop {
		let mut data = Vec::new();
		match source.read_event_into(&mut data)? {
			Event::End(end) if end.name() == name => break,
			Event::End(_) | Event::Eof => return Err(Error::Metadata),
			Event::Start(start) => {
				let (name, _) = parse_name(start.name().as_ref());
				let value = parse_string(source, start.name().as_ref())?;
				if !value.is_empty() {
					metadata.push(Tag { name, value });
				}
			}
			_ => {}
		}
	}
	Ok(())
}

fn parse_properties<R: Read>(
	source: &mut Reader<BufReader<R>>,
	name: QName,
	metadata: &mut Vec<Tag>,
) -> Result<(), Error> {
	loop {
		let mut data = Vec::new();
		match source.read_event_into(&mut data)? {
			Event::End(end) if end.name() == name => break,
			Event::End(_) | Event::Eof => return Err(Error::Metadata),
			Event::Start(start) => {
				let (name, _) = parse_name(start.name().as_ref());
				let value = match name.as_str() {
					"DigSig" | "HeadingPairs" | "HLinks" | "Properties" | "TitlesOfParts" => {
						skip(source, start.name().as_ref())?;
						UNKNOWN.to_string()
					}
					_ => parse_string(source, start.name().as_ref())?,
				};
				if !value.is_empty() {
					metadata.push(Tag { name, value });
				}
			}
			_ => {}
		}
	}
	Ok(())
}

pub fn delete<R: Read + Seek, W: Write + Seek>(
	source: &mut R,
	destination: &mut W,
	code: &str,
) -> Result<(), Error> {
	let mut archive = ZipArchive::new(source)?;
	let mut destination = ZipWriter::new(destination);
	let options = SimpleFileOptions::default()
		.compression_method(CompressionMethod::Stored)
		.last_modified_time(DateTime::default());
	for i in 0..archive.len() {
		let entry = archive.by_index(i)?;
		let name = entry.name();
		match name {
			"[Content_Types].xml" => {
				destination.raw_copy_file_touch(entry, DateTime::default(), None)?;
			}
			"docProps/app.xml" => {
				destination.start_file(name, options)?;
				destination.write_all(br#"<?xml version="1.0" encoding="UTF-8"?><Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"></Properties>"#)?;
			}
			"docProps/core.xml" => {
				destination.start_file(name, options)?;
				destination.write_all(br#"<?xml version="1.0" encoding="UTF-8"?><coreProperties xmlns="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"></coreProperties>"#)?;
			}
			"docProps/custom.xml" => {
				destination.start_file(name, options)?;
				destination.write_all(br#"<?xml version="1.0" encoding="UTF-8"?><Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties"></Properties>"#)?;
			}
			name if name.ends_with(".rels") && !name.starts_with("customXml") => {
				destination.start_file(name, options)?;
				let mut data = Vec::new();
				let mut reader = Reader::from_reader(BufReader::new(entry));
				let mut writer = Writer::new(&mut destination);
				'events: loop {
					let event = reader.read_event_into(&mut data)?;
					match &event {
						Event::Eof => break,
						Event::Empty(element) => {
							if element.name().as_ref() == b"Relationship" {
								for attribute in element.attributes() {
									let attribute = attribute?;
									if attribute.key.as_ref() == b"Type"
										&& attribute.value.ends_with(b"customXml")
									{
										continue 'events;
									}
								}
							}
						}
						_ => {}
					}
					writer.write_event(event)?;
				}
			}
			name if name.starts_with(code) => {
				destination.raw_copy_file_touch(entry, DateTime::default(), None)?;
			}
			_ => {}
		}
	}
	destination.finish()?;
	Ok(())
}
