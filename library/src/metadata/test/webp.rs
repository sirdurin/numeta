use super::{data, error, metadata};
use crate::{
	Tag,
	metadata::webp::{delete, get},
};
use std::io::Cursor;

const BASIC: &[u8] = &[
	b'R', b'I', b'F', b'F', 0x1E, 0x00, 0x00, 0x00, b'W', b'E', b'B', b'P', b'V', b'P', b'8', b'L',
	0x11, 0x00, 0x00, 0x00, 0x2F, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD0, 0xFF, 0xFE, 0xF7, 0xBF, 0xFF,
	0x81, 0x88, 0xE8, 0x7F, 0x00, 0x00,
];

const EXIF: &[u8] = &[
	b'R', b'I', b'F', b'F', 0x46, 0x00, 0x00, 0x00, b'W', b'E', b'B', b'P', b'V', b'P', b'8', b'L',
	0x11, 0x00, 0x00, 0x00, 0x2F, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD0, 0xFF, 0xFE, 0xF7, 0xBF, 0xFF,
	0x81, 0x88, 0xE8, 0x7F, 0x00, 0x00, b'E', b'X', b'I', b'F', 0x1F, 0x00, 0x00, 0x00, b'M', b'M',
	0x00, 0x2A, 0x00, 0x00, 0x00, 0x08, 0x00, 0x01, 0x01, 0x31, 0x00, 0x02, 0x00, 0x00, 0x00, 0x05,
	0x00, 0x00, 0x00, 0x1A, 0x00, 0x00, 0x00, 0x00, b'G', b'I', b'M', b'P', 0x00, 0x00,
];

const XMP: &[u8] = &[
	b'R', b'I', b'F', b'F', 0x56, 0x01, 0x00, 0x00, b'W', b'E', b'B', b'P', b'V', b'P', b'8', b'L',
	0x11, 0x00, 0x00, 0x00, 0x2F, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD0, 0xFF, 0xFE, 0xF7, 0xBF, 0xFF,
	0x81, 0x88, 0xE8, 0x7F, 0x00, 0x00, b'X', b'M', b'P', b' ', 0x30, 0x01, 0x00, 0x00, b'<', b'?',
	b'x', b'p', b'a', b'c', b'k', b'e', b't', b' ', b'b', b'e', b'g', b'i', b'n', b'=', b'"', b'"',
	b' ', b'i', b'd', b'=', b'"', b'W', b'5', b'M', b'0', b'M', b'p', b'C', b'e', b'h', b'i', b'H',
	b'z', b'r', b'e', b'S', b'z', b'N', b'T', b'c', b'z', b'k', b'c', b'9', b'd', b'"', b'?', b'>',
	b'<', b'x', b':', b'x', b'm', b'p', b'm', b'e', b't', b'a', b' ', b'x', b'm', b'l', b'n', b's',
	b':', b'x', b'=', b'"', b'a', b'd', b'o', b'b', b'e', b':', b'n', b's', b':', b'm', b'e', b't',
	b'a', b'/', b'"', b'>', b'<', b'r', b'd', b'f', b':', b'R', b'D', b'F', b' ', b'x', b'm', b'l',
	b'n', b's', b':', b'r', b'd', b'f', b'=', b'"', b'h', b't', b't', b'p', b':', b'/', b'/', b'w',
	b'w', b'w', b'.', b'w', b'3', b'.', b'o', b'r', b'g', b'/', b'1', b'9', b'9', b'9', b'/', b'0',
	b'2', b'/', b'2', b'2', b'-', b'r', b'd', b'f', b'-', b's', b'y', b'n', b't', b'a', b'x', b'-',
	b'n', b's', b'#', b'"', b'>', b'<', b'r', b'd', b'f', b':', b'D', b'e', b's', b'c', b'r', b'i',
	b'p', b't', b'i', b'o', b'n', b' ', b'x', b'm', b'l', b'n', b's', b':', b'x', b'm', b'p', b'=',
	b'"', b'h', b't', b't', b'p', b':', b'/', b'/', b'n', b's', b'.', b'a', b'd', b'o', b'b', b'e',
	b'.', b'c', b'o', b'm', b'/', b'x', b'a', b'p', b'/', b'1', b'.', b'0', b'/', b'"', b'>', b'<',
	b'x', b'm', b'p', b':', b'C', b'r', b'e', b'a', b't', b'e', b'D', b'a', b't', b'e', b'>', b'2',
	b'0', b'2', b'5', b'-', b'0', b'1', b'-', b'2', b'0', b'T', b'1', b'8', b':', b'3', b'0', b':',
	b'0', b'0', b'+', b'0', b'0', b'<', b'/', b'x', b'm', b'p', b':', b'C', b'r', b'e', b'a', b't',
	b'e', b'D', b'a', b't', b'e', b'>', b'<', b'/', b'r', b'd', b'f', b':', b'D', b'e', b's', b'c',
	b'r', b'i', b'p', b't', b'i', b'o', b'n', b'>', b'<', b'/', b'r', b'd', b'f', b':', b'R', b'D',
	b'F', b'>', b'<', b'/', b'x', b':', b'x', b'm', b'p', b'm', b'e', b't', b'a', b'>',
];

const UNKNOWN: &[u8] = &[
	b'R', b'I', b'F', b'F', 0x3A, 0x00, 0x00, 0x00, b'W', b'E', b'B', b'P', b'V', b'P', b'8', b'L',
	0x11, 0x00, 0x00, 0x00, 0x2F, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD0, 0xFF, 0xFE, 0xF7, 0xBF, 0xFF,
	0x81, 0x88, 0xE8, 0x7F, 0x00, 0x00, b'D', b'A', b'T', b'E', 0x13, 0x00, 0x00, 0x00, b'2', b'0',
	b'2', b'5', b'-', b'0', b'1', b'-', b'2', b'0', b' ', b'1', b'8', b':', b'3', b'0', b':', b'0',
	b'0', 0x00,
];

#[test]
fn get_basic() {
	metadata!(BASIC);
}

#[test]
fn delete_basic() {
	data!(BASIC, BASIC);
}

#[test]
fn get_exif() {
	metadata!(EXIF, "Software" => "GIMP");
}

#[test]
fn delete_exif() {
	data!(EXIF, BASIC);
}

#[test]
fn get_xmp() {
	metadata!(XMP, "xmp:CreateDate" => "2025-01-20T18:30:00+00");
}

#[test]
fn delete_xmp() {
	data!(XMP, BASIC);
}

#[test]
fn get_unknown() {
	metadata!(UNKNOWN);
}

#[test]
fn delete_unknown() {
	data!(UNKNOWN, BASIC);
}

#[test]
fn missing_data() {
	error!(&[
		b'R', b'I', b'F', b'F', 0x26, 0x00, 0x00, 0x00, b'W', b'E', b'B', b'P', b'V', b'P', b'8',
		b'L', 0x11, 0x00, 0x00, 0x00, 0x2F, 0x00, 0x00, 0x00, 0x00, 0x07, 0xD0, 0xFF, 0xFE, 0xF7,
		0xBF, 0xFF, 0x81, 0x88, 0xE8, 0x7F, 0x00, 0x00, b'E', b'X', b'I', b'F', 0x1F, 0x00, 0x00,
		0x00,
	]);
}
