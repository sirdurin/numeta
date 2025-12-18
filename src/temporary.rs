use fastrand::alphanumeric;
use std::{
	fs::{File, OpenOptions, remove_file, rename},
	io::{ErrorKind, Result},
	iter::repeat_with,
	path::{Path, PathBuf},
};

pub struct Temporary {
	delete: bool,
	path: PathBuf,
	pub writer: File,
}

impl Temporary {
	pub fn persist<P: AsRef<Path>>(mut self, path: P) -> Result<()> {
		rename(&self.path, path)?;
		self.delete = false;
		Ok(())
	}

	pub fn unique<P: AsRef<Path>>(directory: P) -> Result<Temporary> {
		let mut name = String::with_capacity(10);
		loop {
			let random: String = repeat_with(alphanumeric).take(5).collect();
			name.push('.');
			name.push_str(random.as_str());
			name.push_str(".tmp");
			let path = directory.as_ref().join(&name);
			match OpenOptions::new().create_new(true).write(true).open(&path) {
				Ok(writer) => {
					return Ok(Temporary {
						delete: true,
						path,
						writer,
					});
				}
				Err(error) => {
					if error.kind() != ErrorKind::AlreadyExists {
						return Err(error);
					}
				}
			}
			name.clear();
		}
	}
}

impl Drop for Temporary {
	fn drop(&mut self) {
		if self.delete {
			let _ = remove_file(&self.path);
		}
	}
}
