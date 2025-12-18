use std::{env::args, path::PathBuf};

#[derive(Debug, PartialEq)]
pub struct Options {
	pub delete: bool,
	pub destination: Option<PathBuf>,
	pub source: PathBuf,
}

impl Options {
	pub fn parse() -> Result<Options, ()> {
		let mut arguments = args();
		parse(&mut arguments)
	}
}

fn parse<I: Iterator<Item = String>>(arguments: &mut I) -> Result<Options, ()> {
	let mut delete = false;
	let mut source: Option<PathBuf> = None;
	let mut destination: Option<PathBuf> = None;
	let mut replace = false;
	let mut options = true;
	arguments.next();
	loop {
		match arguments.next().as_deref() {
			Some("-d") => delete = true,
			Some("-r") if delete => replace = true,
			Some("-o") if delete => match arguments.next() {
				Some(value) if !value.is_empty() => destination = Some(PathBuf::from(value)),
				_ => return Err(()),
			},
			Some("--") => options = false,
			Some(value) if options && value.starts_with('-') => return Err(()),
			Some(value) if source.is_none() => source = Some(PathBuf::from(value)),
			None => break,
			_ => return Err(()),
		}
	}
	if source.is_none() || (delete && replace && destination.is_some()) {
		return Err(());
	}
	if delete && replace {
		destination = source.clone();
	}
	return Ok(Options {
		delete,
		source: source.unwrap(),
		destination,
	});
}

#[cfg(test)]
macro_rules! arguments {
	($($value:expr),+) => {{
		let mut values = Vec::new();
		$(
			values.push($value.to_string());
		)+
		values.into_iter()
	}};
}

#[test]
fn no_arguments() {
	assert!(parse(&mut arguments!("numeta")).is_err());
}

#[test]
fn get_one_file() {
	assert_eq!(
		parse(&mut arguments!("numeta", "1.png")),
		Ok(Options {
			delete: false,
			source: PathBuf::from("1.png"),
			destination: None,
		})
	);
}

#[test]
fn get_two_files() {
	assert!(parse(&mut arguments!("numeta", "1.png", "2.png")).is_err(),);
}

#[test]
fn get_dash() {
	assert_eq!(
		parse(&mut arguments!("numeta", "--", "-")),
		Ok(Options {
			delete: false,
			source: PathBuf::from("-"),
			destination: None,
		})
	);
}

#[test]
fn get_unknown_option() {
	assert!(parse(&mut arguments!("numeta", "-r", "1.png")).is_err());
}

#[test]
fn delete_one_file() {
	assert_eq!(
		parse(&mut arguments!("numeta", "-d", "1.png")),
		Ok(Options {
			delete: true,
			destination: None,
			source: PathBuf::from("1.png"),
		})
	);
}

#[test]
fn delete_two_files() {
	assert!(parse(&mut arguments!("numeta", "-d", "1.png", "2.png")).is_err(),);
}

#[test]
fn delete_dash() {
	assert_eq!(
		parse(&mut arguments!("numeta", "-d", "--", "-")),
		Ok(Options {
			delete: true,
			destination: None,
			source: PathBuf::from("-"),
		})
	);
}

#[test]
fn delete_o() {
	assert_eq!(
		parse(&mut arguments!("numeta", "-d", "-o", "2.png", "1.png")),
		Ok(Options {
			delete: true,
			destination: Some(PathBuf::from("2.png")),
			source: PathBuf::from("1.png"),
		})
	);
}

#[test]
fn delete_r() {
	assert_eq!(
		parse(&mut arguments!("numeta", "-d", "-r", "1.png")),
		Ok(Options {
			delete: true,
			destination: Some(PathBuf::from("1.png")),
			source: PathBuf::from("1.png"),
		})
	);
}

#[test]
fn delete_unknown_option() {
	assert!(parse(&mut arguments!("numeta", "-d", "-x", "1.png")).is_err());
}
