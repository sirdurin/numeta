use crate::{
	Error, Tag,
	utilities::stream::{Be, Bytes},
};
use std::{
	io::{ErrorKind, Read, Seek, Write, copy},
	slice,
};

const XING_SIZE: usize = 120;

pub fn get<R: Read + Seek>(source: &mut R) -> Result<Vec<Tag>, Error> {
	let mut metadata = Vec::new();
	loop {
		let Some(header) = header(source)? else {
			return Ok(metadata);
		};
		if header[0] == 0xFF {
			let (size, _) = parse_header(&header)?;
			source.seek_relative(size as i64 - 4)?;
		} else if &header[0..3] == b"ID3" {
			// TODO
			source.seek_relative(2)?;
			let mut data = [0; 4];
			source.read_exact(&mut data)?;
			let size = size(&data);
			source.seek_relative(size as i64)?;
		} else if &header[0..3] == b"TAG" {
			macro_rules! read_tag {
				($size: expr, $name:expr) => {{
					let mut data = [0; $size];
					source.read_exact(&mut data)?;
					let size = data.iter().position(|value| *value == 0).unwrap_or($size);
					if size > 0 {
						metadata.push(Tag {
							name: $name.to_string(),
							value: String::from_utf8_lossy(&data[..size]).to_string(),
						});
					}
				}};
			}
			source.seek_relative(-1)?;
			read_tag!(30, "Title");
			read_tag!(30, "Artist");
			read_tag!(30, "Album");
			read_tag!(4, "Year");
			read_tag!(30, "Comment");
			metadata.push(Tag {
				name: "Genre".to_string(),
				value: genre(Be::u8(source)?),
			});
		} else {
			return Err(Error::File);
		}
	}
}

pub fn delete<R: Read + Seek, W: Write>(source: &mut R, destination: &mut W) -> Result<(), Error> {
	let mut start = true;
	loop {
		let Some(header) = header(source)? else {
			return Ok(());
		};
		if header[0] == 0xFF {
			let (size, position) = parse_header(&header)?;
			if start {
				start = false;
				if size >= XING_SIZE {
					let mut data = [0; 36];
					source.read_exact(&mut data)?;
					let name = &data[position..position + 4];
					if name == b"Info" || name == b"Xing" {
						source.seek_relative(size as i64 - 40)?;
					} else {
						destination.write_all(&header)?;
						destination.write_all(&data)?;
						copy(&mut source.take(size as u64 - 40), destination)?;
					}
					continue;
				}
			}
			destination.write_all(&header)?;
			copy(&mut source.take(size as u64 - 4), destination)?;
		} else if &header[0..3] == b"ID3" {
			source.seek_relative(2)?;
			let mut data = [0; 4];
			source.read_exact(&mut data)?;
			let size = size(&data);
			source.seek_relative(size as i64)?;
		} else if &header[0..3] == b"TAG" {
			break;
		} else {
			return Err(Error::File);
		}
	}

	Ok(())
}

fn header<R: Read>(source: &mut R) -> Result<Option<[u8; 4]>, Error> {
	let mut data = [0; 4];
	if let Err(error) = source.read_exact(slice::from_mut(&mut data[0])) {
		if error.kind() == ErrorKind::UnexpectedEof {
			return Ok(None);
		}
		return Err(Error::Io(error));
	};
	source.read_exact(&mut data[1..4])?;
	Ok(Some(data))
}

fn size(data: &[u8; 4]) -> u32 {
	((data[0] as u32) << 21) + ((data[1] as u32) << 14) + ((data[2] as u32) << 7) + data[3] as u32
}

enum Version {
	V1,
	V2,
}

enum Layer {
	L1,
	L2,
	L3,
}

fn parse_header(header: &[u8; 4]) -> Result<(usize, usize), Error> {
	let version = match header[1] & 0b00011000 {
		0b00011000 => Version::V1,
		0b00010000 => Version::V2,
		_ => return Err(Error::File),
	};

	let is_mono = (header[3] & 0b11000000) == 0b11000000;
	let xing_position = match (&version, is_mono) {
		(Version::V1, false) => 32,
		(Version::V1, true) => 17,
		(Version::V2, false) => 17,
		(Version::V2, true) => 9,
	};

	let layer = match header[1] & 0b00000110 {
		0b00000110 => Layer::L1,
		0b00000100 => Layer::L2,
		0b00000010 => Layer::L3,
		_ => return Err(Error::File),
	};

	let value = header[2] & 0b11110000;
	let bitrate: usize = match (&version, &layer) {
		(Version::V1, Layer::L1) => match value {
			0b00010000 => 32,
			0b00100000 => 64,
			0b00110000 => 96,
			0b01000000 => 128,
			0b01010000 => 160,
			0b01100000 => 192,
			0b01110000 => 224,
			0b10000000 => 256,
			0b10010000 => 288,
			0b10100000 => 320,
			0b10110000 => 352,
			0b11000000 => 384,
			0b11010000 => 416,
			0b11100000 => 448,
			_ => return Err(Error::File),
		},
		(Version::V1, Layer::L2) => match value {
			0b00010000 => 32,
			0b00100000 => 48,
			0b00110000 => 56,
			0b01000000 => 64,
			0b01010000 => 80,
			0b01100000 => 96,
			0b01110000 => 112,
			0b10000000 => 128,
			0b10010000 => 160,
			0b10100000 => 192,
			0b10110000 => 224,
			0b11000000 => 256,
			0b11010000 => 320,
			0b11100000 => 384,
			_ => return Err(Error::File),
		},
		(Version::V1, Layer::L3) => match value {
			0b00010000 => 32,
			0b00100000 => 40,
			0b00110000 => 48,
			0b01000000 => 56,
			0b01010000 => 64,
			0b01100000 => 80,
			0b01110000 => 96,
			0b10000000 => 112,
			0b10010000 => 128,
			0b10100000 => 160,
			0b10110000 => 192,
			0b11000000 => 224,
			0b11010000 => 256,
			0b11100000 => 320,
			_ => return Err(Error::File),
		},
		(Version::V2, Layer::L1) => match value {
			0b00010000 => 32,
			0b00100000 => 48,
			0b00110000 => 56,
			0b01000000 => 64,
			0b01010000 => 80,
			0b01100000 => 96,
			0b01110000 => 112,
			0b10000000 => 128,
			0b10010000 => 144,
			0b10100000 => 160,
			0b10110000 => 176,
			0b11000000 => 192,
			0b11010000 => 224,
			0b11100000 => 256,
			_ => return Err(Error::File),
		},
		(Version::V2, _) => match value {
			0b00010000 => 8,
			0b00100000 => 16,
			0b00110000 => 24,
			0b01000000 => 32,
			0b01010000 => 40,
			0b01100000 => 48,
			0b01110000 => 56,
			0b10000000 => 64,
			0b10010000 => 80,
			0b10100000 => 96,
			0b10110000 => 112,
			0b11000000 => 128,
			0b11010000 => 144,
			0b11100000 => 160,
			_ => return Err(Error::File),
		},
	} * 1000;

	let value = header[2] & 0b00001100;
	let samples: usize = match version {
		Version::V1 => match value {
			0b00000000 => 44100,
			0b00000100 => 48000,
			0b00001000 => 32000,
			_ => return Err(Error::File),
		},
		Version::V2 => match value {
			0b00000000 => 22050,
			0b00000100 => 24000,
			0b00001000 => 16000,
			_ => return Err(Error::File),
		},
	};

	let padding = ((header[2] & 0b00000010) >> 1) as usize;

	let size = match (version, layer) {
		(_, Layer::L1) => 12,
		(_, Layer::L2) => 144,
		(Version::V1, Layer::L3) => 144,
		(Version::V2, Layer::L3) => 72,
	};

	Ok((size * bitrate / samples + padding, xing_position))
}

fn genre(value: u8) -> String {
	match value {
		0 => "Blues",
		1 => "Classic Rock",
		2 => "Country",
		3 => "Dance",
		4 => "Disco",
		5 => "Funk",
		6 => "Grunge",
		7 => "Hip-Hop",
		8 => "Jazz",
		9 => "Metal",
		10 => "New Age",
		11 => "Oldies",
		12 => "Other",
		13 => "Pop",
		14 => "R&B",
		15 => "Rap",
		16 => "Reggae",
		17 => "Rock",
		18 => "Techno",
		19 => "Industrial",
		20 => "Alternative",
		21 => "Ska",
		22 => "Death Metal",
		23 => "Pranks",
		24 => "Soundtrack",
		25 => "Euro-Techno",
		26 => "Ambient",
		27 => "Trip-Hop",
		28 => "Vocal",
		29 => "Jazz & Funk",
		30 => "Fusion",
		31 => "Trance",
		32 => "Classical",
		33 => "Instrumental",
		34 => "Acid",
		35 => "House",
		36 => "Game",
		37 => "Sound Clip",
		38 => "Gospel",
		39 => "Noise",
		40 => "Alternative Rock",
		41 => "Bass",
		42 => "Soul",
		43 => "Punk",
		44 => "Space",
		45 => "Meditative",
		46 => "Instrumental Pop",
		47 => "Instrumental Rock",
		48 => "Ethnic",
		49 => "Gothic",
		50 => "Darkwave",
		51 => "Techno-Industrial",
		52 => "Electronic",
		53 => "Pop-Folk",
		54 => "Eurodance",
		55 => "Dream",
		56 => "Southern Rock",
		57 => "Comedy",
		58 => "Cult",
		59 => "Gangsta",
		60 => "Top 40",
		61 => "Christian Rap",
		62 => "Pop/Funk",
		63 => "Jungle",
		64 => "Native US",
		65 => "Cabaret",
		66 => "New Wave",
		67 => "Psychadelic",
		68 => "Rave",
		69 => "Showtunes",
		70 => "Trailer",
		71 => "Lo-Fi",
		72 => "Tribal",
		73 => "Acid Punk",
		74 => "Acid Jazz",
		75 => "Polka",
		76 => "Retro",
		77 => "Musical",
		78 => "Rock 'n' Roll",
		79 => "Hard Rock",
		80 => "Folk",
		81 => "Folk-Rock",
		82 => "National Folk",
		83 => "Swing",
		84 => "Fast Fusion",
		85 => "Bebop",
		86 => "Latin",
		87 => "Revival",
		88 => "Celtic",
		89 => "Bluegrass",
		90 => "Avantgarde",
		91 => "Gothic Rock",
		92 => "Progressive Rock",
		93 => "Psychedelic Rock",
		94 => "Symphonic Rock",
		95 => "Slow Rock",
		96 => "Big Band",
		97 => "Chorus",
		98 => "Easy Listening",
		99 => "Acoustic",
		100 => "Humour",
		101 => "Speech",
		102 => "Chanson",
		103 => "Opera",
		104 => "Chamber Music",
		105 => "Sonata",
		106 => "Symphony",
		107 => "Booty Bass",
		108 => "Primus",
		109 => "Porn Groove",
		110 => "Satire",
		111 => "Slow Jam",
		112 => "Club",
		113 => "Tango",
		114 => "Samba",
		115 => "Folklore",
		116 => "Ballad",
		117 => "Power Ballad",
		118 => "Rhythmic Soul",
		119 => "Freestyle",
		120 => "Duet",
		121 => "Punk Rock",
		122 => "Drum Solo",
		123 => "A capella",
		124 => "Euro-House",
		125 => "Dance Hall",
		126 => "Goa",
		127 => "Drum & Bass",
		128 => "Club-House",
		129 => "Hardcore Techno",
		130 => "Terror",
		131 => "Indie",
		132 => "BritPop",
		133 => "Negerpunk",
		134 => "Polsk Punk",
		135 => "Beat",
		136 => "Christian Gangsta Rap",
		137 => "Heavy Metal",
		138 => "Black Metal",
		139 => "Crossover",
		140 => "Contemporary Christian",
		141 => "Christian Rock",
		142 => "Merengue",
		143 => "Salsa",
		144 => "Thrash Metal",
		145 => "Anime",
		146 => "Jpop",
		147 => "Synthpop",
		148 => "Abstract",
		149 => "Art Rock",
		150 => "Baroque",
		151 => "Bhangra",
		152 => "Big Beat",
		153 => "Breakbeat",
		154 => "Chillout",
		155 => "Downtempo",
		156 => "Dub",
		157 => "EBM",
		158 => "Eclectic",
		159 => "Electro",
		160 => "Electroclash",
		161 => "Emo",
		162 => "Experimental",
		163 => "Garage",
		164 => "Global",
		165 => "IDM",
		166 => "Illbient",
		167 => "Industro-Goth",
		168 => "Jam Band",
		169 => "Krautrock",
		170 => "Leftfield",
		171 => "Lounge",
		172 => "Math Rock",
		173 => "New Romantic",
		174 => "Nu-Breakz",
		175 => "Post-Punk",
		176 => "Post-Rock",
		177 => "Psytrance",
		178 => "Shoegaze",
		179 => "Space Rock",
		180 => "Trop Rock",
		181 => "World Music",
		182 => "Neoclassical",
		183 => "Audiobook",
		184 => "Audio Theatre",
		185 => "Neue Deutsche Welle",
		186 => "Podcast",
		187 => "Indie Rock",
		188 => "G-Funk",
		189 => "Dubstep",
		190 => "Garage Rock",
		191 => "Psybient",
		_ => "UNKNOWN",
	}
	.to_string()
}
