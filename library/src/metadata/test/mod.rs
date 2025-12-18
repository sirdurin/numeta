mod exif;
mod iptc;
mod jpeg;
mod png;
mod webp;
mod xmp;

macro_rules! data {
	($a:expr, $b:expr) => {
		let mut data = Vec::new();
		assert!(delete(&mut Cursor::new($a), &mut data).is_ok());
		assert_eq!(data, $b);
	};
}

macro_rules! metadata {
	($data:expr) => {
        let data = get(&mut Cursor::new($data));
        assert!(data.is_ok());
		assert!(data.unwrap().is_empty());
	};
	($data:expr, $($name:expr => $value:expr),*) => {
		let mut b = Vec::new();
        $(b.push(Tag { name: $name.to_string(), value: $value.to_string() });)*
        let a = get(&mut Cursor::new($data));
        assert!(a.is_ok());
		assert_eq!(a.unwrap(), b);
	};
}

macro_rules! error {
	($data:expr) => {
		assert!(get(&mut Cursor::new($data)).is_err());
	};
}

pub(self) use data;
pub(self) use error;
pub(self) use metadata;
