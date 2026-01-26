use super::{parse_application, parse_core};
use crate::Tag;
use std::io::{BufReader, Cursor};

macro_rules! ok {
	($function:ident, $data:expr, $($name:expr => $value:expr),*) => {{
		let source = BufReader::new(Cursor::new($data));
		let mut metadata_a = Vec::new();
		let mut metadata_b = Vec::new();
		$(metadata_b.push(Tag { name: $name.to_string(), value: $value.to_string() });)*
		assert!($function(source, &mut metadata_a).is_ok());
		assert_eq!(metadata_a, metadata_b);
	}};
}

#[test]
fn application() {
	let data = br#"
	<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
		<Application>Microsoft Office Word</Application>
		<AppVersion>16.0000</AppVersion>
	</Properties>"#;
	ok!(parse_application, data, "Application" => "Microsoft Office Word", "AppVersion" => "16.0000");
}

#[test]
fn application_no_value() {
	let data = br#"
	<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
		<Application>Microsoft Office Word</Application>
		<AppVersion></AppVersion>
	</Properties>"#;
	ok!(parse_application, data, "Application" => "Microsoft Office Word");
}

#[test]
fn application_unknown_value() {
	let data = br#"
	<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
		<Application>Microsoft Office Word</Application>
		<AppVersion><Unknown>16.0000</Unknown></AppVersion>
	</Properties>"#;
	ok!(parse_application, data, "Application" => "Microsoft Office Word", "AppVersion" => "Unknown");
}

#[test]
fn core() {
	let data = br#"
	<coreProperties
		xmlns="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
		xmlns:dc="http://purl.org/dc/elements/1.1/"
	>
		<dc:creator>Mike Rotch</dc:creator>
		<dc:language>en-US</dc:language>
	</coreProperties>"#;
	ok!(parse_core, data, "creator" => "Mike Rotch", "language" => "en-US");
}

#[test]
fn core_no_value() {
	let data = br#"
	<coreProperties
		xmlns="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
		xmlns:dc="http://purl.org/dc/elements/1.1/"
	>
		<dc:creator></dc:creator>
		<dc:language>en-US</dc:language>
	</coreProperties>"#;
	ok!(parse_core, data, "language" => "en-US");
}

#[test]
fn core_unknown_value() {
	let data = br#"
	<coreProperties
		xmlns="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
		xmlns:dc="http://purl.org/dc/elements/1.1/"
	>
		<dc:creator><Unknown>Mike Rotch</Unknown></dc:creator>
		<dc:language>en-US</dc:language>
	</coreProperties>"#;
	ok!(parse_core, data, "creator" => "Unknown", "language" => "en-US");
}
