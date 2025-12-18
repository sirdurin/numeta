use numeta::{Error, Metadata};
use std::{
	borrow::Cow,
	env::current_dir,
	ffi::{OsStr, OsString},
	fs::{File, OpenOptions},
	io::{BufReader, BufWriter},
	path::{Path, PathBuf},
};

mod options;
mod temporary;
use options::Options;
use temporary::Temporary;

fn main() -> Result<(), Error> {
	let Ok(options) = Options::parse() else {
		println!("Usage:");
		println!("  numeta <file>");
		println!("  numeta -d [-o <file>|-r] <file>");
		println!("");
		println!("Options:");
		println!("  -d   Delete metadata");
		println!("  -o   Write the results to a file");
		println!("  -r   Write the results to the input file");
		return Ok(());
	};
	let source = File::open(&options.source)?;
	let mut source = BufReader::new(source);
	let extension = options.source.extension().and_then(OsStr::to_str);
	let Some(metadata) = Metadata::guess(&mut source, extension)? else {
		eprintln!("File format not supported");
		return Ok(());
	};
	if options.delete {
		let directory = directory(&options.destination);
		let temporary = Temporary::unique(directory)?;
		metadata.delete(&mut source, &mut BufWriter::new(&temporary.writer))?;
		let destination = options
			.destination
			.unwrap_or_else(|| create_from_template(&options.source));
		temporary.persist(destination)?;
	} else {
		for tag in metadata.get(&mut source)? {
			println!("{}", tag);
		}
	}
	Ok(())
}

fn directory(path: &Option<PathBuf>) -> Cow<'_, Path> {
	match path {
		Some(path) => match path.parent() {
			Some(path) => {
				if path.as_os_str().is_empty() {
					Cow::Owned(current_dir().unwrap())
				} else {
					Cow::Borrowed(path)
				}
			}
			None => Cow::Borrowed(path),
		},
		None => Cow::Owned(current_dir().unwrap()),
	}
}

fn create_from_template<P: AsRef<Path>>(template: P) -> PathBuf {
	let template = template.as_ref();
	let extension = template.extension().unwrap();
	let base = template.file_stem().unwrap();
	let mut name = OsString::new();
	name.push(base);
	name.push(".");
	name.push(extension);
	let mut suffix = 1;
	loop {
		if OpenOptions::new()
			.write(true)
			.create_new(true)
			.open(&name)
			.is_ok()
		{
			return PathBuf::from(name);
		}
		suffix += 1;
		name.clear();
		name.push(base);
		name.push("-");
		name.push(suffix.to_string());
		name.push(".");
		name.push(extension);
	}
}
