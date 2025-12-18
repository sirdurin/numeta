use crate::{Error, Tag};
use std::str::{FromStr, from_utf8};
use xmp_toolkit::{IterOptions, XmpMeta};

pub fn get(source: &[u8], metadata: &mut Vec<Tag>) -> Result<(), Error> {
	let data = XmpMeta::from_str(from_utf8(source)?)?;
	let options = IterOptions::default().leaf_nodes_only().omit_qualifiers();
	for property in data.iter(options) {
		metadata.push(Tag {
			name: property.name,
			value: property.value.value,
		});
	}
	Ok(())
}
