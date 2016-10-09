use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::hash::Hasher;
use std::{io, fs};

use walkdir::WalkDir;
use itertools::Itertools;
use twox_hash::XxHash;

/// A file hash.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Hash(pub u64);

/// A cache of files in a directory.
#[derive(Clone, Debug)]
pub struct Cache
{
    /// The base directory of the file cache.
    directory: PathBuf,
    /// The files in the directory, relative to the base directory.
    files: HashMap<PathBuf, File>,
}

/// A file in the cache.
#[derive(Clone, Debug)]
pub struct File
{
    pub path: PathBuf,
    pub version: Hash,
}

/// The status of a file when quering the cache.
#[derive(Copy, Clone, Debug)]
pub enum FileStatus
{
    /// The file is missing - we don't have any versions of it.
    Missing,
    /// We have the file, but a different version.
    DifferentVersion(Hash),
    /// We have an exact match for the file.
    Match,
}

impl Cache
{
    /// Creates a new cache.
    pub fn new(directory: PathBuf) -> Self {
        let mut cache = Cache {
            directory: directory,
            files: HashMap::new(),
        };

        cache.rebuild();

        cache
    }

    /// Queries the cache for the status of a file.
    pub fn query(&self, file: &File) -> FileStatus {
        if let Some(ref cached_file) = self.files.get(&file.path) {
            if cached_file.version == file.version {
                FileStatus::Match
            } else {
                FileStatus::DifferentVersion(cached_file.version)
            }
        } else {
            FileStatus::Missing
        }
    }

    /// Puts a file into the cache.
    pub fn put(&mut self, file: File, data: &[u8]) -> Result<(), io::Error> {
        // Ensure the parent directory exists.
        if let Some(parent_path) = file.path.parent() {
            if !parent_path.exists() { fs::create_dir_all(parent_path)? }
        }

        let mut fs = fs::File::create(&file.path)?;
        fs.write(data)?;

        self.files.insert(file.path.clone(), file);

        Ok(())
    }

    /// Rebuilds the cache.
    pub fn rebuild(&mut self) {
        if !self.directory.exists() {
            fs::create_dir_all(&self.directory).expect("could not create cache directory");
        }

        self.files = WalkDir::new(&self.directory).min_depth(1).into_iter().filter_map(|entry| {
            let entry = entry.unwrap();

            if entry.file_type().is_file() {
                let hash = self::hash_file(entry.path()).expect("could not hash file");

                Some((entry.path().to_owned(), File {
                    path: entry.path().to_owned(),
                    version: hash,
                }))
            } else {
                None
            }
        }).collect();
    }
}

fn hash_file(path: &Path) -> Result<Hash, io::Error> {
    let mut hasher = XxHash::default();
    let file = fs::File::open(path)?;

    for chunk in file.bytes().chunks(4*1024).into_iter() {
        let bytes: Result<Vec<u8>, _> = chunk.collect();
        let bytes = bytes?;

        hasher.write(&bytes);
    }

    Ok(Hash(hasher.finish()))
}

