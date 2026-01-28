use crate::{Error, UNKNOWN};
use quick_xml::{
	Reader,
	events::{BytesStart, Event, attributes::Attribute},
	name::QName,
};
use std::io::BufRead;

#[derive(Debug, Clone)]
pub enum Namespace {
	Dc,
	DcTerms,
	CoreProperties,
	XProperties,
	Unknown,
}

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

pub fn parse_namespace<'a>(attribute: &'a Attribute) -> Option<(&'a [u8], Option<&'a [u8]>)> {
	let mut tokens = attribute.key.as_ref().split(|&character| character == b':');
	if !tokens.nth(0).map(|name| name == b"xmlns").unwrap_or(false) {
		return None;
	}
	Some((&attribute.value, tokens.nth(0)))
}

pub fn parse_start<R: BufRead>(
	source: &mut Reader<R>,
	start: &BytesStart,
	namespaces: &[(Namespace, String)],
) -> Result<(String, String), Error> {
	let (name, prefix) = parse_name(start.name().as_ref());
	let namespace = namespaces
		.iter()
		.find(|namespace| namespace.1 == prefix)
		.map(|namespace| namespace.0.clone())
		.unwrap_or(Namespace::Unknown);
	let value = match (namespace, name.as_bytes()) {
		(Namespace::XProperties, b"DigSig") => {
			skip(source, &start.name())?;
			UNKNOWN.to_string()
		}
		(Namespace::XProperties, b"HLinks") => {
			skip(source, &start.name())?;
			UNKNOWN.to_string()
		}
		(Namespace::XProperties, b"Properties") => {
			skip(source, &start.name())?;
			UNKNOWN.to_string()
		}
		(Namespace::XProperties, b"TitlesOfParts") => {
			skip(source, &start.name())?;
			UNKNOWN.to_string()
		}
		_ => parse_string(source, &start.name())?,
	};
	Ok((name, value))
}

pub fn parse_string<R: BufRead>(source: &mut Reader<R>, name: &QName) -> Result<String, Error> {
	let mut data = Vec::new();
	let mut value = None;
	let mut unknown = false;
	loop {
		match source.read_event_into(&mut data)? {
			Event::End(end) if end.name() == *name => break,
			Event::End(_) | Event::Eof => return Err(Error::Metadata),
			Event::Start(start) => {
				unknown = true;
				skip(source, &start.name())?;
			}
			Event::Text(text) if value.is_none() => {
				value = Some(String::from_utf8_lossy(&text).trim_ascii().to_string());
			}
			_ => {
				unknown = true;
			}
		}
	}
	Ok(if unknown {
		UNKNOWN.to_string()
	} else {
		value.unwrap_or("".to_string())
	})
}

pub fn skip<R: BufRead>(source: &mut Reader<R>, name: &QName) -> Result<(), Error> {
	let mut data = Vec::new();
	loop {
		match source.read_event_into(&mut data)? {
			Event::End(end) if end.name() == *name => break,
			Event::End(_) | Event::Eof => return Err(Error::Metadata),
			Event::Start(start) => skip(source, &start.name())?,
			_ => {}
		}
	}
	Ok(())
}
