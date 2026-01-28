use crate::{
	Error, Tag,
	xml::{Namespace, parse_name, parse_namespace, parse_start, skip},
};
use quick_xml::{events::Event, reader::Reader, writer::Writer};
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
			"docProps/app.xml" => parse_application(entry, &mut metadata)?,
			"docProps/core.xml" => parse_core(entry, &mut metadata)?,
			_ => {}
		}
	}
	Ok(metadata)
}

fn parse_application<R: Read>(source: R, metadata: &mut Vec<Tag>) -> Result<(), Error> {
	let mut data = Vec::new();
	let mut source = Reader::from_reader(BufReader::new(source));
	loop {
		match source.read_event_into(&mut data)? {
			Event::Eof => break,
			Event::Start(start) => {
				let (name, prefix) = parse_name(start.name().as_ref());
				let mut namespaces = [(Namespace::XProperties, "".to_string())];
				for attribute in start.attributes() {
					if let Some((space, code)) = parse_namespace(&attribute?) {
						if space == b"http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" {
							namespaces[0].1 = String::from_utf8_lossy(code.unwrap_or(b"")).to_string();
						}
					}
				}
				if !(name == "Properties" && prefix == namespaces[0].1) {
					skip(&mut source, &start.name())?;
					continue;
				}
				loop {
					let mut data = Vec::new();
					match source.read_event_into(&mut data)? {
						Event::End(end) if end.name() == start.name() => break,
						Event::End(_) | Event::Eof => return Err(Error::Metadata),
						Event::Start(start) => {
							let (name, value) = parse_start(&mut source, &start, &namespaces)?;
							if !value.is_empty() {
								metadata.push(Tag { name, value });
							}
						}
						_ => {}
					}
				}
			}
			_ => {}
		}
	}
	Ok(())
}

fn parse_core<R: Read>(source: R, metadata: &mut Vec<Tag>) -> Result<(), Error> {
	let mut data = Vec::new();
	let mut source = Reader::from_reader(BufReader::new(source));
	loop {
		match source.read_event_into(&mut data)? {
			Event::Eof => break,
			Event::Start(start) => {
				let (name, prefix) = parse_name(start.name().as_ref());
				let mut namespaces = [
					(Namespace::CoreProperties, "".to_string()),
					(Namespace::Dc, "".to_string()),
					(Namespace::DcTerms, "".to_string()),
				];
				for attribute in start.attributes() {
					if let Some((namespace, alias)) = parse_namespace(&attribute?) {
						match namespace {
							b"http://schemas.openxmlformats.org/package/2006/metadata/core-properties" => {
								namespaces[0].1 = String::from_utf8_lossy(alias.unwrap_or(b"")).to_string();
							}
							b"http://purl.org/dc/elements/1.1/" => {
								namespaces[1].1 = String::from_utf8_lossy(alias.unwrap_or(b"")).to_string();
							}
							b"http://purl.org/dc/terms/" => {
								namespaces[2].1 = String::from_utf8_lossy(alias.unwrap_or(b"")).to_string();
							}
							_ => {}
						}
					}
				}
				if !(name == "coreProperties" && prefix == namespaces[0].1) {
					skip(&mut source, &start.name())?;
					continue;
				}
				loop {
					let mut data = Vec::new();
					match source.read_event_into(&mut data)? {
						Event::End(end) if end.name() == start.name() => break,
						Event::End(_) | Event::Eof => return Err(Error::Metadata),
						Event::Start(start) => {
							let (name, value) = parse_start(&mut source, &start, &namespaces)?;
							if !value.is_empty() {
								metadata.push(Tag { name, value });
							}
						}
						_ => {}
					}
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
