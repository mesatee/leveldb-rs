//! An `env` is an abstraction layer that allows the database to run both on different platforms as
//! well as persisting data on disk or in memory.
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::error::{Result, Status};

use std::io::prelude::*;
use std::path::{Path, PathBuf};

use protected_fs::ProtectedFile;

pub trait RandomAccess {
    fn read_at(&self, off: usize, dst: &mut [u8]) -> Result<usize>;
}

#[cfg(unix)]
impl RandomAccess for ProtectedFile {
    fn read_at(&self, off: usize, dst: &mut [u8]) -> Result<usize> {
        self.read_at(off, dst).map_err(|e| Status::from(e))
    }
}

#[cfg(windows)]
impl RandomAccess for File {
    fn read_at(&self, off: usize, dst: &mut [u8]) -> Result<usize> {
        Ok((self as &dyn FileExt).seek_read(dst, off as u64)?)
    }
}

pub struct FileLock {
    pub id: String,
}

pub trait Env {
    fn open_sequential_file(&self, _: &Path) -> Result<Box<dyn Read>>;
    fn open_random_access_file(&self, _: &Path) -> Result<Box<dyn RandomAccess>>;
    fn open_writable_file(&self, _: &Path) -> Result<Box<dyn Write>>;
    fn open_appendable_file(&self, _: &Path) -> Result<Box<dyn Write>>;

    fn exists(&self, _: &Path) -> Result<bool>;
    fn children(&self, _: &Path) -> Result<Vec<PathBuf>>;
    fn size_of(&self, _: &Path) -> Result<usize>;

    fn delete(&self, _: &Path) -> Result<()>;
    fn mkdir(&self, _: &Path) -> Result<()>;
    fn rmdir(&self, _: &Path) -> Result<()>;
    fn rename(&self, _: &Path, _: &Path) -> Result<()>;

    fn lock(&self, _: &Path) -> Result<FileLock>;
    fn unlock(&self, l: FileLock) -> Result<()>;

    fn new_logger(&self, _: &Path) -> Result<Logger>;

    fn micros(&self) -> u64;
    fn sleep_for(&self, micros: u32);
}

pub struct Logger {
    dst: Box<dyn Write>,
}

impl Logger {
    pub fn new(w: Box<dyn Write>) -> Logger {
        Logger { dst: w }
    }

    pub fn log(&mut self, message: &str) {
        let _ = self.dst.write(message.as_bytes());
        let _ = self.dst.write("\n".as_bytes());
    }
}

pub fn path_to_string(p: &Path) -> String {
    p.to_str().map(String::from).unwrap()
}

pub fn path_to_str(p: &Path) -> &str {
    p.to_str().unwrap()
}
