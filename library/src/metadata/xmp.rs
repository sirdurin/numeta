use crate::{Error, Tag};
use quick_xml::{Reader, events::Event, name::QName};

pub fn get(data: &[u8], metadata: &mut Vec<Tag>) -> Result<(), Error> {
	let data = String::from_utf8_lossy(data);
	let mut reader = Reader::from_str(&data);
	loop {
		match reader.read_event()? {
			Event::Start(start) => parse(&mut reader, start.name(), true, metadata)?,
			Event::Eof => break,
			_ => {}
		}
	}
	Ok(())
}

fn parse(
	reader: &mut Reader<&[u8]>,
	name: QName,
	keep: bool,
	metadata: &mut Vec<Tag>,
) -> Result<(), Error> {
	let code = String::from_utf8_lossy(name.as_ref()).to_string();
	let mut tokens = code.splitn(2, ':');
	let code = tokens.nth(1).unwrap_or_else(|| tokens.next().unwrap_or(""));
	let keep = keep
		&& match code {
			"History" | "Manifest" | "Thumbnails" => false,
			_ => true,
		};
	loop {
		match reader.read_event()? {
			Event::End(end) if end.name() == name => break,
			Event::End(_) | Event::Eof => return Err(Error::Metadata),
			Event::Start(start) => parse(reader, start.name(), keep, metadata)?,
			Event::Text(value) if value.trim_ascii().is_empty() => {}
			Event::Text(value) => {
				if keep {
					let name = code.to_string();
					let value = String::from_utf8_lossy(value.as_ref()).to_string();
					metadata.push(Tag { name, value });
				}
			}
			_ => {}
		}
	}
	Ok(())
}
