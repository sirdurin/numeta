use crate::{
	Error, Tag, UNKNOWN,
	utilities::bytes::{Be, Bytes, Le},
};

macro_rules! parse_float {
	($value:expr, $data:ident) => {{
		let numerator = B::$data(&$value[0..]) as f64;
		let denominator = B::$data(&$value[4..]) as f64;
		(numerator / denominator).to_string()
	}};
}

macro_rules! parse_integer {
	($value:expr, $count:expr, $data:ident) => {{
		let mut values = Vec::with_capacity($count);
		for _ in 0..$count {
			values.push(B::$data($value).to_string());
		}
		values.join(" ")
	}};
}

pub fn get(data: &[u8], metadata: &mut Vec<Tag>) -> Result<(), Error> {
	if data.len() < 8 {
		return Err(Error::Metadata);
	}
	match &data[0..2] {
		b"MM" => parse::<Be>(data, metadata),
		b"II" => parse::<Le>(data, metadata),
		_ => return Err(Error::Metadata),
	}
}

fn parse<B: Bytes>(data: &[u8], metadata: &mut Vec<Tag>) -> Result<(), Error> {
	let value = B::u16(&data[2..]);
	if value != 42 {
		return Err(Error::Metadata);
	}
	let mut position = B::u32(&data[4..]) as usize;
	for _ in 0..2 {
		if position == 0 {
			break;
		}
		if position < 8 || position > data.len() {
			return Err(Error::Metadata);
		}
		position = parse_directory::<B>(data, position, names_1, metadata)?;
	}
	Ok(())
}

fn parse_directory<B: Bytes>(
	data: &[u8],
	cursor: usize,
	names: fn(u16) -> String,
	metadata: &mut Vec<Tag>,
) -> Result<usize, Error> {
	if cursor + 2 > data.len() {
		return Err(Error::Metadata);
	}
	let count = B::u16(&data[cursor..]) as usize;
	let mut i = 0;
	while i < count {
		let (name, value) = parse_entry::<B>(&data, cursor + 2 + 12 * i)?;
		i += 1;
		match name {
			34665 => {
				let cursor = value.parse::<u32>()?;
				parse_directory::<B>(data, cursor as usize, names_2, metadata)?;
			}
			34853 => {
				let cursor = value.parse::<u32>()?;
				parse_directory::<B>(data, cursor as usize, names_3, metadata)?;
			}
			_ => {
				let name = names(name);
				metadata.push(Tag { name, value });
			}
		}
	}
	let position = B::u32(&data[cursor + 2 + 12 * i..]) as usize;
	Ok(position)
}

fn parse_entry<B: Bytes>(data: &[u8], cursor: usize) -> Result<(u16, String), Error> {
	if cursor + 12 > data.len() {
		return Err(Error::Metadata);
	}
	let name = B::u16(&data[cursor..]);
	let value_type = B::u16(&data[cursor + 2..]);
	let count = B::u32(&data[cursor + 4..]) as usize;
	let size = count * size(value_type);
	let value = if size > 4 {
		let cursor = &data[cursor + 8..cursor + 12];
		let cursor = B::u32(cursor) as usize;
		if cursor + size > data.len() {
			return Err(Error::Metadata);
		}
		&data[cursor..cursor + size]
	} else {
		&data[cursor + 8..cursor + 8 + size]
	};
	let value = match value_type {
		1 => parse_integer!(value, count, u8),
		2 | 129 => String::from_utf8_lossy(&value[0..count - 1]).to_string(),
		3 => parse_integer!(value, count, u16),
		4 => parse_integer!(value, count, u32),
		5 => parse_float!(value, u32),
		7 => String::from_utf8_lossy(&value[0..4]).to_string(),
		9 => parse_integer!(value, count, i32),
		10 => parse_float!(value, i32),
		_ => "".to_string(),
	};
	Ok((name, value))
}

fn size(data: u16) -> usize {
	match data {
		1 => 1,
		2 | 129 => 1,
		3 => 2,
		4 => 4,
		5 => 8,
		7 => 1,
		9 => 4,
		10 => 8,
		_ => 0,
	}
}

fn names_1(value: u16) -> String {
	match value {
		256 => "ImageWidth",
		257 => "ImageLength",
		258 => "BitsPerSample",
		259 => "Compression",
		262 => "PhotometricInterpretation",
		270 => "ImageDescription",
		271 => "Make",
		272 => "Model",
		273 => "StripOffsets",
		274 => "Orientation",
		277 => "SamplesPerPixel",
		278 => "RowsPerStrip",
		279 => "StripByteCounts",
		282 => "XResolution",
		283 => "YResolution",
		284 => "PlanarConfiguration",
		296 => "ResolutionUnit",
		301 => "TransferFunction",
		305 => "Software",
		306 => "DateTime",
		315 => "Artist",
		318 => "WhitePoint",
		319 => "PrimaryChromaticities",
		513 => "JPEGInterchangeFormat",
		514 => "JPEGInterchangeFormatLength",
		529 => "YCbCrCoefficients",
		530 => "YCbCrSubSampling",
		531 => "YCbCrPositioning",
		532 => "ReferenceBlackWhite",
		33432 => "Copyright",
		_ => UNKNOWN,
	}
	.to_string()
}

fn names_2(value: u16) -> String {
	match value {
		33434 => "ExposureTime",
		33437 => "FNumber",
		34850 => "ExposureProgram",
		34852 => "SpectralSensitivity",
		34855 => "PhotographicSensitivity",
		34856 => "OECF",
		34864 => "SensitivityType",
		34865 => "StandardOutputSensitivity",
		34866 => "RecommendedExposureIndex",
		34867 => "ISOSpeed",
		34868 => "ISOSpeedLatitudeyyy",
		34869 => "ISOSpeedLatitudezzz",
		36864 => "ExifVersion",
		36867 => "DateTimeOriginal",
		36868 => "DateTimeDigitized",
		36880 => "OffsetTime",
		36881 => "OffsetTimeOriginal",
		36882 => "OffsetTimeDigitized",
		37121 => "ComponentsConfiguration",
		37122 => "CompressedBitsPerPixel",
		37377 => "ShutterSpeedValue",
		37378 => "ApertureValue",
		37379 => "BrightnessValue",
		37380 => "ExposureBiasValue",
		37381 => "MaxApertureValue",
		37382 => "SubjectDistance",
		37383 => "MeteringMode",
		37384 => "LightSource",
		37385 => "Flash",
		37386 => "FocalLength",
		37396 => "SubjectArea",
		37500 => "MakerNote",
		37510 => "UserComment",
		37520 => "SubSecTime",
		37521 => "SubSecTimeOriginal",
		37522 => "SubSecTimeDigitized",
		37888 => "Temperature",
		37889 => "Humidity",
		37890 => "Pressure",
		37891 => "WaterDepth",
		37892 => "Acceleration",
		37893 => "CameraElevationAngle",
		40960 => "FlashpixVersion",
		40961 => "ColorSpace",
		40962 => "PixelXDimension",
		40963 => "PixelYDimension",
		40964 => "RelatedSoundFile",
		41483 => "FlashEnergy",
		41484 => "SpatialFrequencyResponse",
		41486 => "FocalPlaneXResolution",
		41487 => "FocalPlaneYResolution",
		41488 => "FocalPlaneResolutionUnit",
		41492 => "SubjectLocation",
		41493 => "ExposureIndex",
		41495 => "SensingMethod",
		41728 => "FileSource",
		41729 => "SceneType",
		41730 => "CFAPattern",
		41985 => "CustomRendered",
		41986 => "ExposureMode",
		41987 => "WhiteBalance",
		41988 => "DigitalZoomRatio",
		41989 => "FocalLengthIn35mmFilm",
		41990 => "SceneCaptureType",
		41991 => "GainControl",
		41992 => "Contrast",
		41993 => "Saturation",
		41994 => "Sharpness",
		41995 => "DeviceSettingDescription",
		41996 => "SubjectDistanceRange",
		42016 => "ImageUniqueID",
		42032 => "CameraOwnerName",
		42033 => "BodySerialNumber",
		42034 => "LensSpecification",
		42035 => "LensMake",
		42036 => "LensModel",
		42037 => "LensSerialNumber",
		42038 => "ImageTitle",
		42039 => "Photographer",
		42040 => "ImageEditor",
		42041 => "CameraFirmware",
		42042 => "RAWDevelopingSoftware",
		42043 => "ImageEditingSoftware",
		42044 => "MetadataEditingSoftware",
		42080 => "CompositeImage",
		42081 => "SourceImageNumberOfCompositeImage",
		42082 => "SourceExposureTimesOfCompositeImage",
		42240 => "Gamma",
		_ => UNKNOWN,
	}
	.to_string()
}

fn names_3(value: u16) -> String {
	match value {
		0 => "GPSVersionID",
		1 => "GPSLatitudeRef",
		2 => "GPSLatitude",
		3 => "GPSLongitudeRef",
		4 => "GPSLongitude",
		5 => "GPSAltitudeRef",
		6 => "GPSAltitude",
		7 => "GPSTimeStamp",
		8 => "GPSSatellites",
		9 => "GPSStatus",
		10 => "GPSMeasureMode",
		11 => "GPSDOP",
		12 => "GPSSpeedRef",
		13 => "GPSSpeed",
		14 => "GPSTrackRef",
		15 => "GPSTrack",
		16 => "GPSImgDirectionRef",
		17 => "GPSImgDirection",
		18 => "GPSMapDatum",
		19 => "GPSDestLatitudeRef",
		20 => "GPSDestLatitude",
		21 => "GPSDestLongitudeRef",
		22 => "GPSDestLongitude",
		23 => "GPSDestBearingRef",
		24 => "GPSDestBearing",
		25 => "GPSDestDistanceRef",
		26 => "GPSDestDistance",
		27 => "GPSProcessingMethod",
		28 => "GPSAreaInformation",
		29 => "GPSDateStamp",
		30 => "GPSDifferential",
		31 => "GPSHPositioningError",
		_ => UNKNOWN,
	}
	.to_string()
}
