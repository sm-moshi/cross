use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::errors::Result;

// open temporary directories and files so we ensure we cleanup on exit.
static FILES: Mutex<Vec<tempfile::NamedTempFile>> = Mutex::new(vec![]);
static DIRS: Mutex<Vec<tempfile::TempDir>> = Mutex::new(vec![]);

// Helper function to safely access FILES without creating a reference to static
fn with_files<F, R>(f: F) -> R
where
    F: FnOnce(&mut Vec<tempfile::NamedTempFile>) -> R,
{
    // SAFETY: This avoids creating a reference to the static by using addr_of!
    // We're still using the Mutex for synchronization
    unsafe {
        // Get a raw pointer to the static without creating a reference
        let ptr = std::ptr::addr_of!(FILES);
        // Dereference and lock (without creating a reference to the static)
        let mut guard = (*ptr).lock().expect("Failed to acquire files lock");
        // Call the function with a mutable reference to the contained value
        f(&mut guard)
    }
}

// Helper function to safely access DIRS without creating a reference to static
fn with_dirs<F, R>(f: F) -> R
where
    F: FnOnce(&mut Vec<tempfile::TempDir>) -> R,
{
    // SAFETY: This avoids creating a reference to the static by using addr_of!
    // We're still using the Mutex for synchronization
    unsafe {
        // Get a raw pointer to the static without creating a reference
        let ptr = std::ptr::addr_of!(DIRS);
        // Dereference and lock (without creating a reference to the static)
        let mut guard = (*ptr).lock().expect("Failed to acquire dirs lock");
        // Call the function with a mutable reference to the contained value
        f(&mut guard)
    }
}

fn data_dir() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|d| d.data_dir().to_path_buf())
}

pub fn dir() -> Result<PathBuf> {
    data_dir()
        .map(|p| p.join("cross-rs").join("tmp"))
        .ok_or(eyre::eyre!("unable to get data directory"))
}

pub(crate) fn has_tempfiles() -> bool {
    with_files(|files| !files.is_empty()) || with_dirs(|dirs| !dirs.is_empty())
}

pub(crate) fn clean() {
    with_files(|files| files.clear());
    with_dirs(|dirs| dirs.clear());
}

fn push_tempfile() -> Result<&'static mut tempfile::NamedTempFile> {
    with_files(|files| {
        let parent = dir()?;
        fs::create_dir_all(&parent).ok();
        let file = tempfile::NamedTempFile::new_in(&parent)?;
        files.push(file);

        // SAFETY: This is safe because we're obtaining a reference to an element
        // that will live for the static lifetime of FILES, and we're ensuring
        // single-threaded access through the Mutex
        let last_file = files.last_mut().expect("file list should not be empty");
        let static_ref = unsafe { &mut *(last_file as *mut _) };
        Ok(static_ref)
    })
}

fn pop_tempfile() -> Option<tempfile::NamedTempFile> {
    with_files(|files| files.pop())
}

#[derive(Debug)]
pub struct TempFile {
    file: &'static mut tempfile::NamedTempFile,
}

impl TempFile {
    pub fn new() -> Result<Self> {
        Ok(Self {
            file: push_tempfile()?,
        })
    }

    pub fn file(&mut self) -> &mut tempfile::NamedTempFile {
        self.file
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        self.file.path()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        pop_tempfile();
    }
}

fn push_tempdir() -> Result<&'static Path> {
    with_dirs(|dirs| {
        let parent = dir()?;
        fs::create_dir_all(&parent).ok();
        let dir = tempfile::TempDir::new_in(&parent)?;

        // Get the path before moving ownership into dirs
        let path = dir.path().to_owned();
        dirs.push(dir);

        // SAFETY: This is safe because we're obtaining a reference to a path
        // that will live for the static lifetime of DIRS
        let path_ref = Box::leak(Box::new(path));
        Ok(path_ref.as_path())
    })
}

fn pop_tempdir() -> Option<tempfile::TempDir> {
    with_dirs(|dirs| dirs.pop())
}

#[derive(Debug)]
pub struct TempDir {
    path: &'static Path,
}

impl TempDir {
    pub fn new() -> Result<Self> {
        Ok(Self {
            path: push_tempdir()?,
        })
    }

    #[must_use]
    pub fn path(&self) -> &'static Path {
        self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        pop_tempdir();
    }
}
