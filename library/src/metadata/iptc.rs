use crate::{
	Error, Tag, UNKNOWN,
	utilities::{
		bytes::{Be, Bytes},
		min,
	},
};

pub fn get(data: &[u8], metadata: &mut Vec<Tag>) -> Result<(), Error> {
	if data.len() < 12 || &data[0..4] != b"8BIM" {
		return Err(Error::Metadata);
	}
	if &data[4..6] != [0x04, 0x04] {
		return Ok(());
	}
	let name = Be::u16(&data[6..8]) as usize;
	let size = Be::u32(&data[name + 8..name + 12]) as usize;
	let mut i = name + 12;
	let max = i + size;
	while i < max {
		if i + 5 > data.len() {
			return Err(Error::Metadata);
		}
		// if data[i] != 0x1C {}
		let size = Be::u16(&data[i + 3..i + 5]) as usize;
		if i + 5 + size > data.len() {
			return Err(Error::Metadata);
		}
		let value = &data[i + 5..i + 5 + size];
		let (name, value) = name_value(value, data[i + 1], data[i + 2]);
		metadata.push(Tag { name, value });
		i = i + 5 + size;
	}
	Ok(())
}

enum Data {
	Number(usize),
	Text,
}

fn name_value(value: &[u8], record: u8, tag: u8) -> (String, String) {
	let (name, data) = match (record, tag) {
		(1, 0) => ("EnvelopeRecordVersion", Data::Number(2)),
		(1, 5) => ("Destination", Data::Text),
		(1, 20) => ("FileFormat", Data::Number(2)),
		(1, 22) => ("FileVersion", Data::Number(2)),
		(1, 30) => ("ServiceIdentifier", Data::Text),
		(1, 40) => ("EnvelopeNumber", Data::Text),
		(1, 50) => ("ProductID", Data::Text),
		(1, 60) => ("EnvelopePriority", Data::Text),
		(1, 70) => ("DateSent", Data::Text),
		(1, 80) => ("TimeSent", Data::Text),
		(1, 90) => ("CodedCharacterSet", Data::Text),
		(1, 100) => ("UniqueObjectName", Data::Text),
		(1, 120) => ("ARMIdentifier", Data::Number(2)),
		(1, 122) => ("ARMVersion", Data::Number(2)),
		(2, 0) => ("ApplicationRecordVersion", Data::Number(2)),
		(2, 3) => ("ObjectTypeReference", Data::Text),
		(2, 4) => ("ObjectAttributeReference", Data::Text),
		(2, 5) => ("ObjectName", Data::Text),
		(2, 7) => ("EditStatus", Data::Text),
		(2, 8) => ("EditorialUpdate", Data::Text),
		(2, 10) => ("Urgency", Data::Text),
		(2, 12) => ("SubjectReference", Data::Text),
		(2, 15) => ("Category", Data::Text),
		(2, 20) => ("SupplementalCategories", Data::Text),
		(2, 22) => ("FixtureIdentifier", Data::Text),
		(2, 25) => ("Keywords", Data::Text),
		(2, 26) => ("ContentLocationCode", Data::Text),
		(2, 27) => ("ContentLocationName", Data::Text),
		(2, 30) => ("ReleaseDate", Data::Text),
		(2, 35) => ("ReleaseTime", Data::Text),
		(2, 37) => ("ExpirationDate", Data::Text),
		(2, 38) => ("ExpirationTime", Data::Text),
		(2, 40) => ("SpecialInstructions", Data::Text),
		(2, 42) => ("ActionAdvised", Data::Text),
		(2, 45) => ("ReferenceService", Data::Text),
		(2, 47) => ("ReferenceDate", Data::Text),
		(2, 50) => ("ReferenceNumber", Data::Text),
		(2, 55) => ("DateCreated", Data::Text),
		(2, 60) => ("TimeCreated", Data::Text),
		(2, 62) => ("DigitalCreationDate", Data::Text),
		(2, 63) => ("DigitalCreationTime", Data::Text),
		(2, 65) => ("OriginatingProgram", Data::Text),
		(2, 70) => ("ProgramVersion", Data::Text),
		(2, 75) => ("ObjectCycle", Data::Text),
		(2, 80) => ("By-line", Data::Text),
		(2, 85) => ("By-lineTitle", Data::Text),
		(2, 90) => ("City", Data::Text),
		(2, 92) => ("Sub-location", Data::Text),
		(2, 95) => ("Province-State", Data::Text),
		(2, 100) => ("Country-PrimaryLocationCode", Data::Text),
		(2, 101) => ("Country-PrimaryLocationName", Data::Text),
		(2, 103) => ("OriginalTransmissionReference", Data::Text),
		(2, 105) => ("Headline", Data::Text),
		(2, 110) => ("Credit", Data::Text),
		(2, 115) => ("Source", Data::Text),
		(2, 116) => ("CopyrightNotice", Data::Text),
		(2, 118) => ("Contact", Data::Text),
		(2, 120) => ("Caption-Abstract", Data::Text),
		(2, 121) => ("LocalCaption", Data::Text),
		(2, 122) => ("Writer-Editor", Data::Text),
		(2, 125) => ("RasterizedCaption", Data::Text),
		(2, 130) => ("ImageType", Data::Text),
		(2, 131) => ("ImageOrientation", Data::Text),
		(2, 135) => ("LanguageIdentifier", Data::Text),
		(2, 150) => ("AudioType", Data::Text),
		(2, 151) => ("AudioSamplingRate", Data::Text),
		(2, 152) => ("AudioSamplingResolution", Data::Text),
		(2, 153) => ("AudioDuration", Data::Text),
		(2, 154) => ("AudioOutcue", Data::Text),
		(2, 184) => ("JobID", Data::Text),
		(2, 185) => ("MasterDocumentID", Data::Text),
		(2, 186) => ("ShortDocumentID", Data::Text),
		(2, 187) => ("UniqueDocumentID", Data::Text),
		(2, 188) => ("OwnerID", Data::Text),
		(2, 200) => ("ObjectPreviewFileFormat", Data::Number(2)),
		(2, 201) => ("ObjectPreviewFileVersion", Data::Number(2)),
		(2, 202) => ("ObjectPreviewData", Data::Text),
		(2, 221) => ("Prefs", Data::Text),
		(2, 225) => ("ClassifyState", Data::Text),
		(2, 228) => ("SimilarityIndex", Data::Text),
		(2, 230) => ("DocumentNotes", Data::Text),
		(2, 231) => ("DocumentHistory", Data::Text),
		(2, 232) => ("ExifCameraInfo", Data::Text),
		(2, 255) => ("CatalogSets", Data::Text),
		(3, 0) => ("NewsPhotoVersion", Data::Number(2)),
		(3, 10) => ("IPTCPictureNumber", Data::Text),
		(3, 20) => ("IPTCImageWidth", Data::Number(2)),
		(3, 30) => ("IPTCImageHeight", Data::Number(2)),
		(3, 40) => ("IPTCPixelWidth", Data::Number(2)),
		(3, 50) => ("IPTCPixelHeight", Data::Number(2)),
		(3, 55) => ("SupplementalType", Data::Number(1)),
		(3, 60) => ("ColorRepresentation", Data::Number(2)),
		(3, 64) => ("InterchangeColorSpace", Data::Number(1)),
		(3, 65) => ("ColorSequence", Data::Number(1)),
		(3, 66) => ("ICC_Profile", Data::Text),
		(3, 70) => ("ColorCalibrationMatrix", Data::Text),
		(3, 80) => ("LookupTable", Data::Text),
		(3, 84) => ("NumIndexEntries", Data::Number(2)),
		(3, 85) => ("ColorPalette", Data::Text),
		(3, 86) => ("IPTCBitsPerSample", Data::Number(1)),
		(3, 90) => ("SampleStructure", Data::Number(1)),
		(3, 100) => ("ScanningDirection", Data::Number(1)),
		(3, 102) => ("IPTCImageRotation", Data::Number(1)),
		(3, 110) => ("DataCompressionMethod", Data::Number(4)),
		(3, 120) => ("QuantizationMethod", Data::Number(1)),
		(3, 125) => ("EndPoints", Data::Text),
		(3, 130) => ("ExcursionTolerance", Data::Number(1)),
		(3, 135) => ("BitsPerComponent", Data::Number(1)),
		(3, 140) => ("MaximumDensityRange", Data::Number(2)),
		(3, 145) => ("GammaCompensatedValue", Data::Number(2)),
		(7, 10) => ("SizeMode", Data::Number(1)),
		(7, 20) => ("MaxSubfileSize", Data::Number(value.len())),
		(7, 90) => ("ObjectSizeAnnounced", Data::Number(value.len())),
		(7, 95) => ("MaximumObjectSize", Data::Number(value.len())),
		(8, 10) => ("SubFile", Data::Number(2)),
		(9, 10) => ("ConfirmedObjectSize", Data::Number(value.len())),
		_ => return (UNKNOWN.to_string(), "".to_string()),
	};
	let name = name.to_string();
	let value = match data {
		Data::Text => String::from_utf8_lossy(value).to_string(),
		Data::Number(size) => {
			let mut number: u64 = 0;
			for i in 0..min!(size, 8) {
				number = (number << 8) | value[i] as u64;
			}
			number.to_string()
		}
	};
	(name, value)
}
